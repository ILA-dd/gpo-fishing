pub mod actions;
pub mod ctx;
pub mod machine;
pub mod session;
pub mod trace;
pub mod watchdog;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use std::thread::JoinHandle;

use crossbeam_channel::Sender;
use parking_lot::{Mutex, RwLock};

use crate::config::{Settings, Store};
use crate::core::platform::Platform;
use crate::core::types::WindowInfo;
use crate::events::{BotEvent, BotState};
use crate::webhook::WebhookQueue;
use ctx::Ctx;

pub struct Bot {
    ctx: Arc<Ctx>,
    thread: Mutex<Option<JoinHandle<()>>>,
    watchdog: Mutex<Option<JoinHandle<()>>>,
    paused: AtomicBool,
}

fn join_unless_self(h: JoinHandle<()>) {
    if h.thread().id() != std::thread::current().id() {
        let _ = h.join();
    }
}

impl Bot {
    pub fn new(
        platform: Platform,
        settings: Arc<RwLock<Settings>>,
        roblox: Arc<RwLock<Option<WindowInfo>>>,
        events: Sender<BotEvent>,
        webhook: Arc<WebhookQueue>,
        store: Arc<Store>,
    ) -> Arc<Self> {
        let ctx = Arc::new(Ctx::new(platform, settings, roblox, events, webhook, store));
        Arc::new(Self {
            ctx,
            thread: Mutex::new(None),
            watchdog: Mutex::new(None),
            paused: AtomicBool::new(false),
        })
    }

    pub fn ctx(&self) -> &Arc<Ctx> {
        &self.ctx
    }

    pub fn is_running(&self) -> bool {
        self.ctx.running.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub fn state(&self) -> BotState {
        self.ctx.state()
    }

    pub fn start(self: &Arc<Self>) {
        if self.is_running() {
            return;
        }
        let resume = self.paused.swap(false, Ordering::SeqCst);
        if resume {
            self.ctx.session.lock().resume();
        } else {
            let mut sess = self.ctx.session.lock();
            *sess = sess.next_session();
        }
        self.ctx.emit_stats();
        self.spawn_loop(resume);
        self.spawn_watchdog();
        self.ctx.log_info(if resume { "Resumed" } else { "Started" });
    }

    pub fn pause(&self) {
        if !self.is_running() {
            return;
        }
        self.paused.store(true, Ordering::SeqCst);
        self.ctx.running.store(false, Ordering::SeqCst);
        self.ctx.set_state(BotState::Paused, None);
        self.ctx.log_info("Paused");
        self.halt();
        self.ctx.session.lock().pause();
        self.ctx.emit_stats();
    }

    pub fn stop(&self) {
        let was = self.is_running() || self.is_paused();
        self.paused.store(false, Ordering::SeqCst);
        self.ctx.running.store(false, Ordering::SeqCst);
        self.ctx.set_state(BotState::Stopped, None);
        if was {
            self.ctx.log_info("Stopped");
        }
        self.halt();
        self.ctx.session.lock().pause();
        self.ctx.emit_stats();
    }

    pub fn toggle(self: &Arc<Self>) {
        if self.is_running() {
            self.pause();
        } else {
            self.start();
        }
    }

    fn halt(&self) {
        self.ctx.running.store(false, Ordering::SeqCst);
        if let Some(h) = self.watchdog.lock().take() {
            join_unless_self(h);
        }
        if let Some(h) = self.thread.lock().take() {
            join_unless_self(h);
        }
        self.ctx.release_mouse();
    }

    fn spawn_loop(&self, skip_setup: bool) {
        self.ctx.running.store(true, Ordering::SeqCst);
        self.ctx.touch();
        let ctx = Arc::clone(&self.ctx);
        let h = std::thread::Builder::new()
            .name("bot-loop".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    machine::run(&ctx, skip_setup);
                }));
                ctx.release_mouse();
                if result.is_err() {
                    ctx.log_error("Bot loop panicked; stopped");
                    ctx.running.store(false, Ordering::SeqCst);
                    ctx.set_state(BotState::Stopped, Some("panic".into()));
                }
            })
            .expect("spawn bot loop");
        *self.thread.lock() = Some(h);
    }

    fn spawn_watchdog(self: &Arc<Self>) {
        if !self.ctx.settings.read().watchdog.enabled {
            return;
        }
        let ctx = Arc::clone(&self.ctx);
        let weak: Weak<Bot> = Arc::downgrade(self);
        let on_stuck: Box<dyn FnOnce(String) + Send> = Box::new(move |reason| {
            if let Some(bot) = weak.upgrade() {
                bot.restart_loop(&reason);
            }
        });
        let h = std::thread::Builder::new()
            .name("bot-watchdog".into())
            .spawn(move || watchdog::run(&ctx, on_stuck))
            .expect("spawn watchdog");
        *self.watchdog.lock() = Some(h);
    }

    pub fn restart_loop(self: &Arc<Self>, reason: &str) {
        let attempt = {
            let mut s = self.ctx.session.lock();
            s.restarts += 1;
            s.restarts
        };
        let (max, backoff) = {
            let s = self.ctx.settings.read();
            (s.watchdog.max_restarts, s.watchdog.restart_backoff_s)
        };
        self.ctx.emit(BotEvent::Recovery { attempt, reason: reason.to_string() });
        self.ctx.webhook.recovery(attempt, reason);
        if attempt > max {
            self.ctx.log_error(&format!("Restart limit reached ({max}); stopping"));
            self.stop();
            return;
        }
        self.ctx.log_warn(&format!("Restarting loop #{attempt}: {reason}"));
        self.ctx.set_state(BotState::Recovering, Some(reason.to_string()));
        self.ctx.running.store(false, Ordering::SeqCst);
        if let Some(h) = self.watchdog.lock().take() {
            join_unless_self(h);
        }
        if let Some(h) = self.thread.lock().take() {
            join_unless_self(h);
        }
        self.ctx.release_mouse();
        std::thread::sleep(std::time::Duration::from_secs_f32(backoff));
        self.spawn_loop(true);
        self.spawn_watchdog();
    }
}

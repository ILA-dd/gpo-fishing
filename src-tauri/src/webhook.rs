use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};
use parking_lot::RwLock;
use serde_json::{json, Value};

use crate::config::Settings;
use crate::core::fruit::{DropInfo, SpawnInfo};
use crate::events::Stats;

const COLOR_BLUE: u32 = 0x3B82F6;
const COLOR_PURPLE: u32 = 0x8B5CF6;
const COLOR_GOLD: u32 = 0xF59E0B;
const COLOR_GREEN: u32 = 0x22C55E;
const COLOR_RED: u32 = 0xEF4444;

pub struct WebhookQueue {
    tx: Option<Sender<Value>>,
    settings: Option<Arc<RwLock<Settings>>>,
}

impl WebhookQueue {
    pub fn disabled() -> Self {
        Self { tx: None, settings: None }
    }

    pub fn start(settings: Arc<RwLock<Settings>>) -> Arc<Self> {
        let (tx, rx) = unbounded::<Value>();
        let s2 = Arc::clone(&settings);
        std::thread::Builder::new()
            .name("webhook".into())
            .spawn(move || worker(rx, s2))
            .expect("spawn webhook worker");
        Arc::new(Self { tx: Some(tx), settings: Some(settings) })
    }

    fn enabled(&self) -> bool {
        self.settings
            .as_ref()
            .map(|s| {
                let s = s.read();
                s.webhook.enabled && !s.webhook.url.trim().is_empty()
            })
            .unwrap_or(false)
    }

    fn send(&self, embed: Value) {
        if !self.enabled() {
            return;
        }
        if let Some(tx) = &self.tx {
            let _ = tx.send(json!({ "embeds": [embed] }));
        }
    }

    pub fn send_raw_now(url: &str, embed: Value) -> Result<(), String> {
        post(url, &json!({ "embeds": [embed] })).map(|_| ())
    }

    pub fn test(&self) -> Result<(), String> {
        let url = self
            .settings
            .as_ref()
            .map(|s| s.read().webhook.url.clone())
            .unwrap_or_default();
        if url.trim().is_empty() {
            return Err("Webhook URL is empty".into());
        }
        Self::send_raw_now(
            &url,
            embed("Webhook connected", "GPO Autofish can reach this channel.", COLOR_GREEN, vec![]),
        )
    }

    pub fn progress(&self, st: Stats) {
        self.send(embed(
            "Fishing progress",
            "",
            COLOR_BLUE,
            vec![
                field("Fish", &st.fish.to_string()),
                field("Fruits", &st.fruits.to_string()),
                field("Runtime", &fmt_runtime(st.runtime_s)),
                field("Success", &format!("{:.0}%", st.success_rate * 100.0)),
            ],
        ));
    }

    pub fn fruit_drop(&self, d: &DropInfo) {
        let (title, color) = if d.is_legendary {
            ("Legendary devil fruit caught", COLOR_GOLD)
        } else {
            ("Devil fruit caught", COLOR_PURPLE)
        };
        let desc = match &d.name {
            Some(n) => format!("You fished up **{n}**."),
            None => format!("You fished up a devil fruit. The name could not be read.\n`{}`", d.text),
        };
        self.send(embed(title, &desc, color, vec![]));
    }

    pub fn spawn(&self, info: &SpawnInfo) {
        let at = info.location.as_deref().map(|l| format!(" at **{l}**")).unwrap_or_default();
        let (title, desc, color) = match &info.name {
            Some(n) => ("Devil fruit spawned", format!("**{n}** has spawned{at}."), COLOR_PURPLE),
            None => ("Devil fruit spawned", format!("A devil fruit has spawned{at}. The name could not be read.\n`{}`", info.text), COLOR_BLUE),
        };
        self.send(embed(title, &desc, color, vec![]));
    }

    pub fn purchase(&self, amount: u32) {
        self.send(embed("Bait purchased", &format!("Bought {amount} bait."), COLOR_GREEN, vec![]));
    }

    pub fn recovery(&self, attempt: u32, reason: &str) {
        let flag = self.settings.as_ref().map(|s| s.read().webhook.recovery).unwrap_or(false);
        if !flag {
            return;
        }
        self.send(embed("Recovery", reason, COLOR_RED, vec![field("Attempt", &attempt.to_string())]));
    }
}

fn embed(title: &str, desc: &str, color: u32, fields: Vec<Value>) -> Value {
    json!({
        "title": title,
        "description": desc,
        "color": color,
        "fields": fields,
        "footer": { "text": "GPO Autofish v4" },
        "timestamp": chrono_now(),
    })
}

fn field(name: &str, value: &str) -> Value {
    json!({ "name": name, "value": value, "inline": true })
}

fn fmt_runtime(s: u64) -> String {
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

fn chrono_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86400;
    let rem = secs % 86400;
    let (y, m, d) = civil_from_days(days as i64);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, (rem % 3600) / 60, rem % 60)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn post(url: &str, body: &Value) -> Result<u16, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.post(url).json(body).send().map_err(|e| e.to_string())?;
    let status = resp.status();
    if status.is_success() {
        Ok(status.as_u16())
    } else {
        Err(format!("Discord returned {}", status.as_u16()))
    }
}

fn worker(rx: Receiver<Value>, settings: Arc<RwLock<Settings>>) {
    for body in rx.iter() {
        let url = settings.read().webhook.url.clone();
        if url.trim().is_empty() {
            continue;
        }
        let mut delay = Duration::from_secs(1);
        for attempt in 0..3 {
            match post(&url, &body) {
                Ok(_) => break,
                Err(e) => {
                    tracing::warn!("webhook attempt {} failed: {e}", attempt + 1);
                    std::thread::sleep(delay);
                    delay *= 2;
                }
            }
        }
    }
}

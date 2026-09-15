use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::protocol::xtest::ConnectionExt as _;
use x11rb::CURRENT_TIME;

use super::x11::X11Connection;
use crate::core::platform::Input;
use crate::core::types::{Key, MouseButton, PxPoint, PxRect};

const KEY_PRESS: u8 = 2;
const KEY_RELEASE: u8 = 3;
const BUTTON_PRESS: u8 = 4;
const BUTTON_RELEASE: u8 = 5;
const MOTION_NOTIFY: u8 = 6;

const XK_BACK_SPACE: u32 = 0xff08;
const XK_RETURN: u32 = 0xff0d;
const XK_ESCAPE: u32 = 0xff1b;
const XK_DELETE: u32 = 0xffff;
const XK_SHIFT_L: u32 = 0xffe1;
const XK_CONTROL_L: u32 = 0xffe3;

/// Synthetic mouse and keyboard input through the server-side XTEST extension.
///
/// XTEST is available on normal Xorg and Xwayland installations. It is more
/// reliable than shelling out to xdotool and lets the macro keep its current
/// no-extra-process input timing.
pub struct XTestInput {
    x11: Option<X11Connection>,
}

impl XTestInput {
    pub fn new() -> Self {
        match X11Connection::open() {
            Ok(x11) => {
                let available = x11
                    .connection
                    .lock()
                    .xtest_get_version(2, 2)
                    .map(|cookie| cookie.reply().is_ok())
                    .unwrap_or(false);
                if !available {
                    tracing::warn!(
                        "the X11 XTEST extension is unavailable; synthetic input will not work"
                    );
                }
                Self {
                    x11: available.then_some(x11),
                }
            }
            Err(err) => {
                tracing::warn!("X11 input is unavailable: {err}");
                Self { x11: None }
            }
        }
    }

    fn fake_input(&self, event_type: u8, detail: u8, point: Option<PxPoint>) -> bool {
        let Some(x11) = &self.x11 else { return false };
        let connection = x11.connection.lock();
        let (x, y) = point
            .map(|p| (clamp_i16(p.x), clamp_i16(p.y)))
            .unwrap_or((0, 0));
        connection
            .xtest_fake_input(event_type, detail, CURRENT_TIME, x11.root, x, y, 0)
            .and_then(|_| connection.flush())
            .is_ok()
    }

    fn keycode_for(&self, keysym: u32) -> Option<(u8, bool)> {
        let x11 = self.x11.as_ref()?;
        let connection = x11.connection.lock();
        let setup = connection.setup();
        let first = setup.min_keycode;
        let count = setup.max_keycode.checked_sub(first)?.saturating_add(1);
        let mapping = connection
            .get_keyboard_mapping(first, count)
            .ok()?
            .reply()
            .ok()?;
        let per_keycode = usize::from(mapping.keysyms_per_keycode);
        if per_keycode == 0 {
            return None;
        }
        mapping
            .keysyms
            .chunks(per_keycode)
            .enumerate()
            .find_map(|(index, keysyms)| {
                keysyms
                    .iter()
                    .position(|&symbol| symbol == keysym)
                    .map(|level| {
                        // Core X11 maps an unshifted and shifted symbol in alternating
                        // slots. Group switching is intentionally left to the user's
                        // active keyboard layout; GPO's configured keys are ASCII.
                        (first.saturating_add(index as u8), level % 2 == 1)
                    })
            })
    }

    fn send_key(&self, key: Key, down: bool) {
        let Some((keycode, needs_shift)) = self.keycode_for(keysym(key)) else {
            tracing::warn!("cannot map X11 key {:?}", key);
            return;
        };
        let shift = needs_shift.then(|| self.keycode_for(XK_SHIFT_L)).flatten();
        if down {
            if let Some((shift_code, _)) = shift {
                self.fake_input(KEY_PRESS, shift_code, None);
            }
            self.fake_input(KEY_PRESS, keycode, None);
        } else {
            self.fake_input(KEY_RELEASE, keycode, None);
            if let Some((shift_code, _)) = shift {
                self.fake_input(KEY_RELEASE, shift_code, None);
            }
        }
    }
}

impl Default for XTestInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for XTestInput {
    fn move_to(&self, p: PxPoint) {
        self.fake_input(MOTION_NOTIFY, 0, Some(p));
    }

    fn button(&self, button: MouseButton, down: bool) {
        let detail = match button {
            MouseButton::Left => 1,
            MouseButton::Right => 3,
        };
        self.fake_input(
            if down { BUTTON_PRESS } else { BUTTON_RELEASE },
            detail,
            None,
        );
    }

    fn key(&self, key: Key, down: bool) {
        self.send_key(key, down);
    }

    fn wheel(&self, delta: i32) {
        if delta == 0 {
            return;
        }
        let detail = if delta > 0 { 4 } else { 5 };
        let steps = (delta.unsigned_abs() / 120).max(1);
        for _ in 0..steps {
            self.fake_input(BUTTON_PRESS, detail, None);
            self.fake_input(BUTTON_RELEASE, detail, None);
        }
    }

    fn type_text(&self, text: &str) {
        for character in text.chars() {
            self.send_key(Key::Char(character), true);
            self.send_key(Key::Char(character), false);
        }
    }

    fn cursor(&self) -> PxPoint {
        let Some(x11) = &self.x11 else {
            return PxPoint::default();
        };
        let connection = x11.connection.lock();
        connection
            .query_pointer(x11.root)
            .ok()
            .and_then(|cookie| cookie.reply().ok())
            .map(|reply| PxPoint {
                x: i32::from(reply.root_x),
                y: i32::from(reply.root_y),
            })
            .unwrap_or_default()
    }

    fn screen(&self) -> PxRect {
        let Some(x11) = &self.x11 else {
            return PxRect::default();
        };
        let connection = x11.connection.lock();
        connection
            .get_geometry(x11.root)
            .ok()
            .and_then(|cookie| cookie.reply().ok())
            .map(|geometry| PxRect {
                x: 0,
                y: 0,
                w: i32::from(geometry.width),
                h: i32::from(geometry.height),
            })
            .unwrap_or_default()
    }
}

fn keysym(key: Key) -> u32 {
    match key {
        Key::Backspace => XK_BACK_SPACE,
        Key::Delete => XK_DELETE,
        Key::Enter => XK_RETURN,
        Key::Escape => XK_ESCAPE,
        Key::Control => XK_CONTROL_L,
        Key::Char(character) if character as u32 <= 0xff => character as u32,
        // X11's Unicode keysym encoding supports non-ASCII text on layouts
        // that expose it, while leaving normal GPO hotkeys on their keymap.
        Key::Char(character) => 0x0100_0000 | character as u32,
    }
}

fn clamp_i16(value: i32) -> i16 {
    value.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_special_and_unicode_keysyms() {
        assert_eq!(keysym(Key::Enter), XK_RETURN);
        assert_eq!(keysym(Key::Char('a')), u32::from(b'a'));
        assert_eq!(keysym(Key::Char('Ж')), 0x0100_0416);
    }
}

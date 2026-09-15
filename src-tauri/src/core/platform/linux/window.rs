use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ClientMessageData, ClientMessageEvent, ConnectionExt as _, EventMask,
    InputFocus, MapState,
};
use x11rb::CURRENT_TIME;

use super::x11::X11Connection;
use crate::core::platform::GameWindow;
use crate::core::types::{PxRect, WindowInfo};

/// Locates Roblox through the EWMH client list and asks the window manager to
/// focus it. The fallback tree scan supports lightweight X11 window managers
/// that do not publish `_NET_CLIENT_LIST`.
pub struct X11Window {
    x11: Option<X11Connection>,
    name: String,
}

impl X11Window {
    pub fn new(name: &str) -> Self {
        match X11Connection::open() {
            Ok(x11) => Self {
                x11: Some(x11),
                name: name.to_ascii_lowercase(),
            },
            Err(err) => {
                tracing::warn!("X11 window discovery is unavailable: {err}");
                Self {
                    x11: None,
                    name: name.to_ascii_lowercase(),
                }
            }
        }
    }

    fn roblox_window(&self, connection: &impl Connection, root: u32) -> Option<u32> {
        let ewmh_clients = atom(connection, b"_NET_CLIENT_LIST")
            .and_then(|clients| window_list(connection, root, clients));
        let candidates = ewmh_clients.unwrap_or_else(|| {
            connection
                .query_tree(root)
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .map(|reply| reply.children)
                .unwrap_or_default()
        });
        candidates
            .into_iter()
            .find(|&window| self.matches(connection, window))
    }

    fn matches(&self, connection: &impl Connection, window: u32) -> bool {
        let title = atom(connection, b"_NET_WM_NAME")
            .and_then(|property| property_text(connection, window, property))
            .or_else(|| property_text(connection, window, AtomEnum::WM_NAME.into()))
            .unwrap_or_default()
            .to_ascii_lowercase();
        let class = property_text(connection, window, AtomEnum::WM_CLASS.into())
            .unwrap_or_default()
            .to_ascii_lowercase();
        matches_game_client(&title, &class, &self.name)
    }
}

/// `Sober` is the native Linux Roblox client. Its X11 class is stable across
/// package sources (`sober` / `org.vinegarhq.Sober`), unlike its short title.
/// Keep the recognition class-based so a browser tab mentioning Roblox does
/// not become a capture/input target.
fn matches_game_client(title: &str, class: &str, roblox_name: &str) -> bool {
    if class.contains("org.vinegarhq.sober") || class.split('\0').any(|part| part == "sober") {
        return true;
    }

    class.contains(roblox_name) || (class.is_empty() && title.contains(roblox_name))
}

impl GameWindow for X11Window {
    fn find(&self) -> Option<WindowInfo> {
        let x11 = self.x11.as_ref()?;
        let connection = x11.connection.lock();
        let window = self.roblox_window(&*connection, x11.root)?;
        let attrs = connection
            .get_window_attributes(window)
            .ok()?
            .reply()
            .ok()?;
        let geometry = connection.get_geometry(window).ok()?.reply().ok()?;
        if attrs.map_state != MapState::VIEWABLE || geometry.width == 0 || geometry.height == 0 {
            return None;
        }
        let origin = connection
            .translate_coordinates(window, x11.root, 0, 0)
            .ok()?
            .reply()
            .ok()?;
        let active = atom(&*connection, b"_NET_ACTIVE_WINDOW")
            .and_then(|property| window_list(&*connection, x11.root, property))
            .and_then(|windows| windows.into_iter().next());
        Some(WindowInfo {
            client: PxRect {
                x: i32::from(origin.dst_x),
                y: i32::from(origin.dst_y),
                w: i32::from(geometry.width),
                h: i32::from(geometry.height),
            },
            is_foreground: active == Some(window),
            visible: true,
            // X11 reports physical pixel geometry; the rest of the app uses
            // pixels directly, so 96 matches the Windows baseline.
            dpi: 96,
        })
    }

    fn focus(&self) -> bool {
        let Some(x11) = &self.x11 else { return false };
        let connection = x11.connection.lock();
        let Some(window) = self.roblox_window(&*connection, x11.root) else {
            return false;
        };
        let Some(active_atom) = atom(&*connection, b"_NET_ACTIVE_WINDOW") else {
            // Tiny/window-manager-free X11 sessions may not implement EWMH.
            // In that case the core protocol is the best available focus path.
            return connection
                .set_input_focus(InputFocus::PARENT, window, CURRENT_TIME)
                .and_then(|_| connection.flush())
                .is_ok();
        };
        let event = ClientMessageEvent::new(
            32,
            window,
            active_atom,
            ClientMessageData::from([1, CURRENT_TIME, 0, 0, 0]),
        );
        connection
            .send_event(
                false,
                x11.root,
                EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
                event,
            )
            .and_then(|_| connection.flush())
            .is_ok()
    }
}

fn atom(connection: &impl Connection, name: &[u8]) -> Option<Atom> {
    connection
        .intern_atom(false, name)
        .ok()?
        .reply()
        .ok()
        .map(|reply| reply.atom)
}

fn window_list(connection: &impl Connection, window: u32, property: Atom) -> Option<Vec<u32>> {
    connection
        .get_property(false, window, property, AtomEnum::WINDOW, 0, u32::MAX)
        .ok()?
        .reply()
        .ok()?
        .value32()
        .map(|values| values.collect())
}

fn property_text(connection: &impl Connection, window: u32, property: Atom) -> Option<String> {
    let reply = connection
        .get_property(false, window, property, AtomEnum::ANY, 0, 4096)
        .ok()?
        .reply()
        .ok()?;
    if reply.value.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&reply.value).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::matches_game_client;

    #[test]
    fn recognizes_roblox_and_sober_game_clients() {
        assert!(matches_game_client("Roblox", "roblox\\0Roblox", "roblox"));
        assert!(matches_game_client(
            "Sober",
            "sober\\0org.vinegarhq.sober",
            "roblox"
        ));
    }

    #[test]
    fn does_not_treat_a_browser_tab_as_the_game_client() {
        assert!(!matches_game_client(
            "Grand Piece Online | Play on Roblox",
            "firefox\\0Firefox",
            "roblox"
        ));
    }
}

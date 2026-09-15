use parking_lot::Mutex;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::Window;
use x11rb::rust_connection::RustConnection;

/// One serialised connection to the current X server.
///
/// X11 requests are ordered, and the platform traits may be called from bot,
/// command, and watcher threads. A mutex avoids interleaving requests on the
/// same socket while still allowing each backend to own its own connection.
pub struct X11Connection {
    pub connection: Mutex<RustConnection>,
    pub root: Window,
}

impl X11Connection {
    pub fn open() -> Result<Self, String> {
        let (connection, screen_num) = x11rb::connect(None).map_err(|err| err.to_string())?;
        let root = connection
            .setup()
            .roots
            .get(screen_num)
            .map(|screen| screen.root)
            .ok_or_else(|| "X11 server returned no default screen".to_string())?;
        Ok(Self {
            connection: Mutex::new(connection),
            root,
        })
    }
}

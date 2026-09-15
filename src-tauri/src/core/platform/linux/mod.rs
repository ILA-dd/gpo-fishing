//! Linux platform implementation.
//!
//! This backend intentionally targets a real X11 server (including an X11
//! session on a compositor). Wayland deliberately does not expose global
//! screen capture or synthetic input through this protocol; see the README
//! for the X11-session requirement.

pub mod capture;
pub mod input;
pub mod ocr;
pub mod window;

mod x11;

#[cfg(test)]
mod tests {
    use super::{capture::X11Capture, input::XTestInput, ocr::TesseractOcr};
    use crate::core::platform::{Capture, Input, Ocr};
    use crate::core::types::PxRect;

    /// Does not need Roblox, but does need the interactive X11 session used by
    /// the application. It never emits input: only XTEST capability, desktop
    /// geometry, a one-pixel capture, and Tesseract's language probe are used.
    #[test]
    #[ignore = "requires an interactive X11 session with Tesseract installed"]
    fn x11_backend_smoke() {
        let input = XTestInput::new();
        assert!(
            !input.screen().is_empty(),
            "XTEST or the X11 root is unavailable"
        );

        let frame = X11Capture::new()
            .grab(PxRect {
                x: 0,
                y: 0,
                w: 1,
                h: 1,
            })
            .expect("X11 root capture failed");
        assert_eq!(frame.rgba.len(), 4);

        assert!(
            TesseractOcr::new().available(),
            "Tesseract English OCR is unavailable"
        );
    }
}

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};

use crate::core::platform::{Ocr, PlatformError, Result};
use crate::core::types::Frame;

/// Tesseract command-line OCR for Linux.
///
/// Calling the maintained `tesseract` executable keeps native C/C++ bindings
/// out of the Rust build. Arch's `tesseract` and `tesseract-data-eng` packages
/// supply the executable and language data; `GPO_TESSERACT` can point to a
/// non-standard executable for portable installations.
pub struct TesseractOcr {
    executable: PathBuf,
    available: OnceLock<bool>,
}

impl TesseractOcr {
    pub fn new() -> Self {
        let executable = std::env::var_os("GPO_TESSERACT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("tesseract"));
        Self {
            executable,
            available: OnceLock::new(),
        }
    }

    fn probe(&self) -> bool {
        let output = Command::new(&self.executable).arg("--list-langs").output();
        match output {
            Ok(output) if output.status.success() => {
                let languages = String::from_utf8_lossy(&output.stdout);
                let available = languages.lines().any(|language| language.trim() == "eng");
                if !available {
                    tracing::warn!(
                        "Tesseract is installed but its English language data is missing"
                    );
                }
                available
            }
            Ok(output) => {
                tracing::warn!(
                    "Tesseract probe failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                );
                false
            }
            Err(err) => {
                tracing::warn!("Tesseract is unavailable: {err}");
                false
            }
        }
    }

    fn is_available(&self) -> bool {
        *self.available.get_or_init(|| self.probe())
    }
}

impl Default for TesseractOcr {
    fn default() -> Self {
        Self::new()
    }
}

impl Ocr for TesseractOcr {
    fn available(&self) -> bool {
        self.is_available()
    }

    fn read(&self, frame: &Frame) -> Result<String> {
        if !self.is_available() {
            return Err(PlatformError::OcrUnavailable);
        }
        if frame.w < 8 || frame.h < 8 {
            return Ok(String::new());
        }
        if frame.rgba.len() != frame.w.saturating_mul(frame.h).saturating_mul(4) {
            return Err(PlatformError::Ocr("invalid RGBA frame".into()));
        }

        let mut png = Vec::new();
        PngEncoder::new(&mut png)
            .write_image(
                &frame.rgba,
                frame.w as u32,
                frame.h as u32,
                ColorType::Rgba8.into(),
            )
            .map_err(|err| PlatformError::Ocr(format!("PNG encoding failed: {err}")))?;

        let mut child = Command::new(&self.executable)
            .args(["stdin", "stdout", "--psm", "6", "-l", "eng"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| PlatformError::Ocr(format!("could not start Tesseract: {err}")))?;
        child
            .stdin
            .take()
            .ok_or_else(|| PlatformError::Ocr("could not open Tesseract input".into()))?
            .write_all(&png)
            .map_err(|err| {
                PlatformError::Ocr(format!("could not send image to Tesseract: {err}"))
            })?;
        let output = child
            .wait_with_output()
            .map_err(|err| PlatformError::Ocr(format!("could not read Tesseract output: {err}")))?;
        if !output.status.success() {
            return Err(PlatformError::Ocr(format!(
                "Tesseract exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_binary_is_reported_as_unavailable() {
        let ocr = TesseractOcr {
            executable: PathBuf::from("gpo-tesseract-does-not-exist"),
            available: OnceLock::new(),
        };
        assert!(!ocr.available());
    }
}

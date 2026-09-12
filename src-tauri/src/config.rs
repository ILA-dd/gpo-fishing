use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::controller::ControlGains;
use crate::core::fruit::Lexicon;
use crate::core::types::{PxRect, RelPoint, RelRect};
use crate::core::vision::Palette;

pub const SETTINGS_VERSION: u32 = 9;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Regions {
    pub bar: RelRect,
    pub drop: RelRect,
}

impl Default for Regions {
    fn default() -> Self {
        Self {
            bar: RelRect { x: 0.546, y: 0.322, w: 0.121, h: 0.436 },
            drop: RelRect { x: 0.369, y: 0.052, w: 0.259, h: 0.113 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Points {
    pub fishing: RelPoint,
    pub purchase: [Option<RelPoint>; 3],
    pub fruit: [Option<RelPoint>; 2],
    pub bait: [Option<RelPoint>; 2],
}

impl Default for Points {
    fn default() -> Self {
        Self {
            fishing: RelPoint { x: 0.5, y: 0.33 },
            purchase: [None, None, None],
            fruit: [None, None],
            bait: [None, None],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Keys {
    pub rod: char,
    pub fruit_slot_1: char,
    pub fruit_slot_2: char,
    pub shop: char,
}

impl Default for Keys {
    fn default() -> Self {
        Self { rod: '1', fruit_slot_1: '2', fruit_slot_2: '3', shop: 'e' }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Fishing {
    pub control: ControlGains,
    pub palette: Palette,
    pub scan_timeout_s: f32,
    pub track_timeout_s: f32,
    pub min_track_s: f32,
    pub bite_confirm_frames: u32,
    pub lost_frames: u32,
    pub wait_after_catch_s: f32,
    pub cast_hold_ms: u32,
    pub scan_hz: u32,
    pub track_hz: u32,
    pub trace: bool,
}

impl Default for Fishing {
    fn default() -> Self {
        Self {
            control: ControlGains::default(),
            palette: Palette::default(),
            scan_timeout_s: 15.0,
            track_timeout_s: 30.0,
            min_track_s: 0.8,
            bite_confirm_frames: 2,
            lost_frames: 4,
            wait_after_catch_s: 1.0,
            cast_hold_ms: 1000,
            scan_hz: 15,
            track_hz: 60,
            trace: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Features {
    pub auto_zoom: bool,
    pub auto_mouse_position: bool,
    pub auto_bait: bool,
    pub fruit_storage: bool,
    pub auto_purchase: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Purchase {
    pub amount: u32,
    pub every_n_catches: u32,
    pub hold_shop_key_ms: u32,
    pub after_key_ms: u32,
    pub click_delay_ms: u32,
    pub after_type_ms: u32,
}

impl Default for Purchase {
    fn default() -> Self {
        Self {
            amount: 100,
            every_n_catches: 10,
            hold_shop_key_ms: 3000,
            after_key_ms: 2000,
            click_delay_ms: 1000,
            after_type_ms: 1500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Zoom {
    pub out_steps: u32,
    pub in_steps: u32,
    pub step_delay_ms: u32,
    pub sequence_delay_ms: u32,
}

impl Default for Zoom {
    fn default() -> Self {
        Self { out_steps: 10, in_steps: 8, step_delay_ms: 100, sequence_delay_ms: 500 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FruitStorage {
    pub key_settle_ms: u32,
    pub click_settle_ms: u32,
    pub dialog_wait_ms: u32,
    pub after_drop_ms: u32,
}

impl Default for FruitStorage {
    fn default() -> Self {
        Self { key_settle_ms: 500, click_settle_ms: 500, dialog_wait_ms: 800, after_drop_ms: 1200 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OcrSettings {
    pub spawn_check_interval_s: f32,
    pub spawn_cooldown_s: f32,
    pub post_catch_reads: u32,
    pub post_catch_read_gap_ms: u32,
}

impl Default for OcrSettings {
    fn default() -> Self {
        Self {
            spawn_check_interval_s: 4.0,
            spawn_cooldown_s: 900.0,
            post_catch_reads: 3,
            post_catch_read_gap_ms: 250,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Webhook {
    pub url: String,
    pub enabled: bool,
    pub progress_every_n: u32,
    pub progress: bool,
    pub fruit_drop: bool,
    pub spawn: bool,
    pub purchase: bool,
    pub recovery: bool,
    pub legendary_only: bool,
}

impl Default for Webhook {
    fn default() -> Self {
        Self {
            url: String::new(),
            enabled: false,
            progress_every_n: 10,
            progress: true,
            fruit_drop: true,
            spawn: true,
            purchase: true,
            recovery: true,
            legendary_only: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hotkeys {
    pub toggle: String,
    pub overlay: String,
    pub quit: String,
    pub hide_hud: String,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self { toggle: "F1".into(), overlay: "F2".into(), quit: "F3".into(), hide_hud: "F4".into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Ui {
    pub theme: String,
    pub hud_offset: RelPoint,
    pub hud_visible: bool,
    pub panel_offset: RelPoint,
    pub panel_size: [u32; 2],
    pub log_level: String,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            hud_offset: RelPoint { x: 0.5, y: 0.0 },
            hud_visible: true,
            panel_offset: RelPoint { x: 1.0, y: 0.5 },
            panel_size: [440, 640],
            log_level: "info".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Watchdog {
    pub enabled: bool,
    pub heartbeat_timeout_s: f32,
    pub max_restarts: u32,
    pub restart_backoff_s: f32,
}

impl Default for Watchdog {
    fn default() -> Self {
        Self { enabled: true, heartbeat_timeout_s: 30.0, max_restarts: 5, restart_backoff_s: 2.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub version: u32,
    pub regions: Regions,
    pub points: Points,
    pub keys: Keys,
    pub fishing: Fishing,
    pub features: Features,
    pub purchase: Purchase,
    pub zoom: Zoom,
    pub fruit_storage: FruitStorage,
    pub ocr: OcrSettings,
    pub lexicon: Lexicon,
    pub webhook: Webhook,
    pub hotkeys: Hotkeys,
    pub ui: Ui,
    pub watchdog: Watchdog,
    pub auto_update: bool,
}

impl Settings {
    pub fn fruit_alerts(&self) -> bool {
        self.webhook.enabled && (self.webhook.spawn || self.webhook.fruit_drop)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            regions: Regions::default(),
            points: Points::default(),
            keys: Keys::default(),
            fishing: Fishing::default(),
            features: Features::default(),
            purchase: Purchase::default(),
            zoom: Zoom::default(),
            fruit_storage: FruitStorage::default(),
            ocr: OcrSettings::default(),
            lexicon: Lexicon::default(),
            webhook: Webhook::default(),
            hotkeys: Hotkeys::default(),
            ui: Ui::default(),
            watchdog: Watchdog::default(),
            auto_update: true,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid preset name")]
    BadName,
}

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.dir.join("logs")
    }

    pub fn purge_trace_frames(&self) -> u64 {
        let traces = self.logs_dir().join("traces");
        let Ok(entries) = fs::read_dir(&traces) else { return 0 };
        let mut freed = 0u64;
        for e in entries.flatten() {
            let path = e.path();
            if path.is_dir() {
                if let Ok(files) = fs::read_dir(&path) {
                    freed += files.flatten().filter_map(|f| f.metadata().ok()).map(|m| m.len()).sum::<u64>();
                }
                let _ = fs::remove_dir_all(&path);
            }
        }
        freed
    }

    fn settings_path(&self) -> PathBuf {
        self.dir.join("settings.json")
    }

    fn presets_dir(&self) -> PathBuf {
        self.dir.join("presets")
    }

    fn stats_path(&self) -> PathBuf {
        self.dir.join("stats.json")
    }

    pub fn load_stats(&self) -> crate::events::Lifetime {
        fs::read_to_string(self.stats_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save_stats(&self, l: &crate::events::Lifetime) -> Result<(), ConfigError> {
        fs::create_dir_all(&self.dir)?;
        write_atomic(&self.stats_path(), &serde_json::to_vec_pretty(l)?)
    }

    pub fn load(&self) -> Settings {
        let mut settings = match fs::read_to_string(self.settings_path()) {
            Ok(s) => serde_json::from_str::<Settings>(&s).unwrap_or_else(|e| {
                tracing::warn!("settings parse failed ({e}); using defaults");
                Settings::default()
            }),
            Err(_) => Settings::default(),
        };
        if settings.version < SETTINGS_VERSION {
            if settings.version < 6 && settings.fishing.control.physics.calibrated_at == 0 {
                settings.fishing.control = crate::core::controller::ControlGains::default();
            }
            if settings.version < 8 {
                let old = settings.points.purchase;
                settings.points.purchase = [old[2], old[1], None];
            }
            if settings.version < 9 {
                settings.fishing.trace = false;
            }
            settings.version = SETTINGS_VERSION;
            let _ = self.save(&settings);
        }
        settings
    }

    pub fn save(&self, s: &Settings) -> Result<(), ConfigError> {
        fs::create_dir_all(&self.dir)?;
        write_atomic(&self.settings_path(), &serde_json::to_vec_pretty(s)?)
    }

    pub fn list_presets(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Ok(rd) = fs::read_dir(self.presets_dir()) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().is_some_and(|x| x == "json") {
                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                        out.push(stem.to_string());
                    }
                }
            }
        }
        out.sort();
        out
    }

    fn preset_path(&self, name: &str) -> Result<PathBuf, ConfigError> {
        let ok = !name.is_empty()
            && name.len() <= 64
            && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ');
        if !ok {
            return Err(ConfigError::BadName);
        }
        Ok(self.presets_dir().join(format!("{name}.json")))
    }

    pub fn save_preset(&self, name: &str, s: &Settings) -> Result<(), ConfigError> {
        fs::create_dir_all(self.presets_dir())?;
        write_atomic(&self.preset_path(name)?, &serde_json::to_vec_pretty(s)?)
    }

    pub fn load_preset(&self, name: &str) -> Result<Settings, ConfigError> {
        let s = fs::read_to_string(self.preset_path(name)?)?;
        Ok(serde_json::from_str(&s)?)
    }

    pub fn delete_preset(&self, name: &str) -> Result<(), ConfigError> {
        fs::remove_file(self.preset_path(name)?)?;
        Ok(())
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), ConfigError> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct LegacyArea {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Debug, Deserialize)]
struct LegacyLayout {
    area: Option<LegacyArea>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct LegacySettings {
    auto_purchase_enabled: bool,
    auto_purchase_amount: u32,
    loops_per_purchase: u32,
    point_coords: std::collections::HashMap<String, [i32; 2]>,
    fruit_coords: std::collections::HashMap<String, [i32; 2]>,
    fishing_location: Option<[i32; 2]>,
    fruit_storage_enabled: bool,
    fruit_storage_key: String,
    fruit_storage_key_2: String,
    rod_key: String,
    auto_bait_enabled: bool,
    top_bait_coords: Option<[i32; 2]>,
    top_bait_coords_2: Option<[i32; 2]>,
    kp: f32,
    kd: f32,
    scan_timeout: f32,
    wait_after_loss: f32,
    webhook_url: String,
    webhook_enabled: bool,
    webhook_interval: u32,
    layout_settings: std::collections::HashMap<String, LegacyLayout>,
    zoom_settings: std::collections::HashMap<String, serde_json::Value>,
}

pub fn import_legacy(json: &str, roblox: PxRect) -> Result<Settings, ConfigError> {
    let l: LegacySettings = serde_json::from_str(json)?;
    let mut s = Settings::default();
    let rel = |p: [i32; 2]| crate::core::types::PxPoint { x: p[0], y: p[1] }.to_rel(&roblox);
    let rect = |a: &LegacyArea| PxRect { x: a.x, y: a.y, w: a.width, h: a.height }.to_rel(&roblox);

    if let Some(a) = l.layout_settings.get("bar").and_then(|x| x.area.as_ref()) {
        s.regions.bar = rect(a);
    }
    if let Some(a) = l.layout_settings.get("drop").and_then(|x| x.area.as_ref()) {
        s.regions.drop = rect(a);
    }
    if let Some(p) = l.fishing_location {
        s.points.fishing = rel(p);
    }
    s.points.purchase = [
        l.point_coords.get("3").map(|p| rel(*p)),
        l.point_coords.get("2").map(|p| rel(*p)),
        None,
    ];
    s.points.fruit[0] = l.fruit_coords.get("fruit_point").map(|p| rel(*p));
    s.points.fruit[1] = l.fruit_coords.get("fruit_point_2").map(|p| rel(*p));
    s.points.bait[0] = l.top_bait_coords.map(rel);
    s.points.bait[1] = l.top_bait_coords_2.map(rel);
    if let Some(c) = l.rod_key.chars().next() {
        s.keys.rod = c;
    }
    if let Some(c) = l.fruit_storage_key.chars().next() {
        s.keys.fruit_slot_1 = c;
    }
    if let Some(c) = l.fruit_storage_key_2.chars().next() {
        s.keys.fruit_slot_2 = c;
    }
    s.features.auto_purchase = l.auto_purchase_enabled;
    s.features.fruit_storage = l.fruit_storage_enabled;
    s.features.auto_bait = l.auto_bait_enabled;
    if l.auto_purchase_amount > 0 {
        s.purchase.amount = l.auto_purchase_amount;
    }
    if l.loops_per_purchase > 0 {
        s.purchase.every_n_catches = l.loops_per_purchase;
    }
    if l.scan_timeout > 0.0 {
        s.fishing.scan_timeout_s = l.scan_timeout;
    }
    if l.wait_after_loss > 0.0 {
        s.fishing.wait_after_catch_s = l.wait_after_loss;
    }
    s.webhook.url = l.webhook_url;
    s.webhook.enabled = l.webhook_enabled;
    if l.webhook_interval > 0 {
        s.webhook.progress_every_n = l.webhook_interval;
    }
    let zoom = &l.zoom_settings;
    s.features.auto_zoom = zoom.get("auto_zoom_enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    s.features.auto_mouse_position =
        zoom.get("auto_mouse_position_enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    if let Some(n) = zoom.get("zoom_in_steps").and_then(|v| v.as_u64()) {
        s.zoom.in_steps = n as u32;
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_defaults() {
        let s = Settings::default();
        let j = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&j).unwrap();
        assert_eq!(back.version, SETTINGS_VERSION);
        assert_eq!(back.keys.rod, '1');
    }

    #[test]
    fn partial_json_fills_defaults() {
        let s: Settings = serde_json::from_str(r#"{"features":{"auto_bait":true}}"#).unwrap();
        assert!(s.features.auto_bait);
        assert_eq!(s.purchase.amount, 100);
    }

    #[test]
    fn legacy_import_converts_to_relative() {
        let legacy = r#"{
          "fishing_location":[960,360],
          "point_coords":{"1":[801,943],"2":[962,941],"3":[1113,943]},
          "layout_settings":{"bar":{"area":{"x":1049,"y":348,"width":233,"height":471}}},
          "rod_key":"1","fruit_storage_key":"2","fruit_storage_key_2":"3",
          "kp":0.2,"kd":0.4,"webhook_url":"https://x","webhook_enabled":true
        }"#;
        let s = import_legacy(legacy, PxRect { x: 0, y: 0, w: 1920, h: 1080 }).unwrap();
        assert!((s.points.fishing.x - 0.5).abs() < 0.001);
        assert!(s.points.purchase[0].is_some());
        assert!(s.points.purchase[1].is_some());
        assert!((s.regions.bar.w - 233.0 / 1920.0).abs() < 0.001);
        assert!(s.webhook.enabled);
    }
}

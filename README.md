[![Discord](https://img.shields.io/badge/Discord-Join%20Server-7289da?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/unPZxXAtfb)

# GPO Autofish — Linux/X11 Fork

> **This repository is a fork of [arielldev/gpo-fishing](https://github.com/arielldev/gpo-fishing).** It preserves the v4.x interface and bot logic, keeps the Windows implementation intact, and adds a native backend for **Arch Linux running an X11 desktop session**.
>
> The Linux backend finds and focuses the Roblox/Sober window, captures the game area, uses Tesseract for OCR, and sends mouse/keyboard actions through X11 XTEST. It is intended for a real **X11/Xorg** login; Wayland is not supported for capture and input automation.

# 🎣 GPO Autofish v4.0 - GUIDE

**💬 Join our Discord server:** https://discord.gg/unPZxXAtfb

## 🆕 What's New in v4.0?

**Complete Rewrite - Native, Fast & Tiny:**

- ⚡ **Native desktop app** - Rewritten in Rust with Tauri. Windows and Linux/X11 builds, no Python
- 🧠 **Built-in text recognition** - Windows uses its native OCR; Linux uses the system Tesseract package
- 📌 **HUD pill + tray icon** - A small always-on-top pill sits on the Roblox window. No big window in the way
- 📐 **Resolution independent** - Every area and click point is saved relative to the Roblox window
- 🖱️ **On-screen editor** - Draw the bar and drop message areas directly over the game. Bar area can be auto-detected
- 👀 **Live preview** - Thumbnail and match score for the bar area so you know it works before you start
- 🧭 **Step-by-step guide** - The Setup page walks you from an empty hotbar to your first catch
- 🔄 **Imports v3 settings** - Loads your old `default_settings.json`

## What is this?

This is the **open-source version** of the GPO fishing macro that everyone uses. Unlike the closed-source version that gets flagged as a virus and isn't trustworthy, this version is:

- ✅ **Fully open source** - You can see and verify all the code
- ✅ **No viruses** - Clean, transparent, and safe
- ✅ **Improved** - Better features and reliability
- ✅ **Community-driven** - Open for contributions and review

The original closed-source macro is sketchy and often flagged by antivirus software because you can't verify what it's actually doing. This open-source version solves that problem.

**🛡️ Concerned about safety?** The whole source is here. Build it yourself with the steps below and compare. Antivirus heuristics dislike programs that send mouse input and read the screen; that is what a fishing macro does.

---

**Features:**

- **🎣 Fishing System** - Automatic fish detection and tracking with a physics-based controller
- **🍎 Devil Fruit Detection** - OCR-powered detection of devil fruit drops with keyword matching
- **🌟 Fruit Spawn Alerts** - Detects and webhooks when devil fruits spawn with exact fruit name recognition
- **📦 Auto Fruit Storage** - Automatically stores devil fruits in your fruit slots when detected
- **🔔 Discord Webhook Alerts** - Notifications for devil fruit catches, world spawns, purchases and recoveries
- **🛒 Auto-Purchase** - Configurable bait purchasing every X fish
- **🪱 Auto Bait** - Re-selects your bait before every cast
- **🎯 Auto Setup** - Zoom control and cast positioning
- **🛟 Watchdog** - Restarts a stuck loop on its own
- **💾 Presets** - Save and load full settings snapshots
- **⬆️ Auto Update** - Updates itself from GitHub Releases
- **⌨️ Global hotkey support** (F1/F2/F3/F4, all rebindable)
- 🐧 **Linux/X11 backend** - Native Roblox discovery, screen capture, XTEST mouse/keyboard input and Tesseract OCR

## 🚀 Key Features

### 🍎 Devil Fruit Detection

- **OCR Detection**: Detects devil fruit drops using the platform OCR backend (Windows OCR or Tesseract on Linux)
- **Spawn Detection**: Detects when devil fruits spawn in the world (all 33 GPO fruits)
- **Fuzzy Matching**: Handles OCR errors with a similarity threshold
- **Auto Storage**: Automatically stores caught fruits into two hotbar slots and re-equips the rod
- **Webhook Alerts**: Discord notifications for catches and spawns

### 🎯 Auto Setup

- **Zoom Control**: Automatically zooms out/in for fishing
- **Cast Positioning**: Moves to the casting position (auto center or a custom point)
- **Menu Clearing**: Right-clicks to clear menus

### 🛒 Auto-Purchase

- **Configurable Intervals**: Buy bait every X fish caught
- **Point System**: Holds the shop key next to the bait barrel, then clicks Confirm, Quantity and an optional Cancel point
- **Auto-save**: Settings persist between sessions

### ⚡ Performance

- **Fast Detection**: Bar tracking runs in microseconds, not Python pixel loops
- **Tiny Footprint**: One small installer, no runtime to install
- **Logging**: Live activity feed in the Dashboard plus a log file in the data folder

## Installation

### 🚀 Easy Installation (Recommended)

1. **Download** the latest `GPO Autofish_x.y.z_x64-setup.exe` from Releases
2. **Run it** - No admin needed
3. **Launch GPO Autofish** - The panel opens; the HUD appears once Roblox is running

Requires Windows 10 1809 or newer. Windows OCR needs an English language pack, which is present on nearly every install. The Setup page tells you if it is missing.

### 🐧 Arch Linux — X11 (supported Linux target)

The Linux port targets a real **X11 session** and keeps the same UI, saved settings, overlay editor and bot logic as v4.x. It uses X11/EWMH to locate and focus Roblox, X11 `GetImage` for capture, XTEST for mouse/keyboard input, and `tesseract` for OCR.

Wayland is intentionally not supported for the macro backend: its permission model does not allow an ordinary application to globally capture another application's pixels or inject input. Log into an **Xorg / X11** desktop session before launching the app. Xwayland inside a Wayland session is not a substitute when Roblox itself is a native Wayland window.

Install the required Arch packages:

```bash
sudo pacman -Syu --needed base-devel git nodejs npm rustup \
  webkit2gtk-4.1 gtk3 librsvg libayatana-appindicator \
  tesseract tesseract-data-eng xorg-xdpyinfo
rustup default stable
```

Clone and start the development build:

```bash
git clone https://github.com/ILA-dd/gpo-fishing.git
cd gpo-fishing
npm install
npm run tauri dev
```

Create an optimized package:

```bash
npm run tauri build
```

The unbundled Linux executable is written to `src-tauri/target/release/gpo-autofish`; Tauri bundles are written below `src-tauri/target/release/bundle/`. The default configuration requests all bundles, so Arch users can run the AppImage or package the release binary in an Arch package. The build itself does not need Wine.

Before reporting a Linux capture/input problem, confirm that the session and required services are visible:

```bash
printf 'session: %s\n' "$XDG_SESSION_TYPE"
xdpyinfo -queryExtensions | grep XTEST
tesseract --list-langs | grep '^eng$'
```

`session: x11`, an `XTEST` line, and `eng` are expected. If Tesseract is installed outside `PATH`, launch the app with `GPO_TESSERACT=/absolute/path/to/tesseract npm run tauri dev`.

You can also run the non-destructive backend smoke check before opening Roblox. It verifies XTEST, X11 root capture, and the English Tesseract language without emitting any mouse or keyboard input:

```bash
cargo test --manifest-path src-tauri/Cargo.toml x11_backend_smoke -- --ignored
```

### 🔧 Build the installer yourself

Requirements: [Node.js 20+](https://nodejs.org) and [Rust](https://rustup.rs). WebView2 is already on Windows 11.

1. **Download the repository** as ZIP and extract it, or `git clone https://github.com/ILA-dd/gpo-fishing.git`
2. **Double-click `MakeItExe.bat`** - It installs packages, builds the app and opens the folder with the installer
3. **Run the installer** it produced, same as the one from Releases

Auto-update checks GitHub Releases on launch and can be turned off in Settings.

## 🎮 Quick Start Guide

### Before you start

- **Rod in slot 1**: Put your fishing rod in the first hotbar slot (key `1`)
- **Empty inventory**: Clear everything else out of your hotbar so fruits and bait land where the bot expects them

### First Time Setup

1. **Launch**: Open Roblox, join GPO, then open GPO Autofish. The Setup page shows the window as detected
2. **Bar area**: Cast once by hand. When the blue bar shows, open Setup › Fishing bar area and press **Auto-detect**. The thumbnail turns green when matched
3. **Drop message area**: Draw it over the popup at the top middle of the screen where "New Item <Fruit>" and "A Fruit has spawned at Place" appear. Press **Read now** to confirm the OCR reads it
4. **Rod key**: Slot `1`
5. **Enable Features**: Turn on auto bait, auto buy, fruit storage and webhooks in the Features page. Each one lists its steps, keys and points. Each **Pick** opens a crosshair over Roblox
6. **Fish**: Press **F1** or the HUD play button

### Devil Fruit Storage Setup

1. **Enable Store fruits** in the Features page
2. **Set Fruit Keys**: Choose the two hotbar slots the fruit is moved through
3. **Set Fruit Point**: Pick the Store button that appears after switching to a fruit slot
4. **Set Rod Key**: Slot `1` holds your fishing rod (Setup › Rod key)
5. **Set Bait Point**: Pick the top bait in the rod menu (Features › Auto bait)

### Auto-Purchase Setup

1. **Stand next to the bait barrel** on the dock before starting
2. **Enable Auto buy bait** in the Features page and follow the numbered steps
3. **Shop key**: The bot holds it until the barrel's shop dialog opens
4. **Confirm / Quantity / Cancel**: Pick each button in the shop. Cancel is optional
5. **Amount and interval**: How much bait to type and how many casts between purchases

### Discord Webhook Setup

1. **Create Webhook**: In your Discord server → Channel Settings → Integrations → Webhooks
2. **Copy URL**: Paste the webhook URL in Features › Discord and press **Test**
3. **Configure Alerts**:
   - 🍎 Devil Fruit Catch Alerts - Notifications when you catch a fruit while fishing
   - 🌟 Devil Fruit Spawn Alerts - Notifications when fruits spawn in the world (with exact fruit name). Turning this on makes the bot read the drop message area between casts
   - 🐟 Fish Progress Updates - Regular progress reports
   - 🛒 Auto Purchase Alerts - Bait purchase confirmations
   - 🛟 Recovery Alerts - When the watchdog restarts or gives up
4. **Set Interval**: Choose how often to send fish progress updates

### Hotkeys

- **F1**: Start/Pause fishing loop
- **F2**: Edit areas on screen
- **F3**: Emergency stop and exit
- **F4**: Hide/show the HUD
- **Note**: All hotkeys work without admin privileges and can be rebound in Settings

### Performance Tips

- **Long Sessions**: Close the panel; the HUD and tray icon keep running
- **Webhook Monitoring**: Use Discord alerts for fruit spawns and catches instead of watching the screen
- **OCR Optimization**: Make the drop message area cover the whole popup for better fruit detection
- **Spawn Detection**: The bot detects all 33 GPO devil fruits automatically using fuzzy matching

---

## 🔧 Troubleshooting

### Runtime Issues

- **HUD not showing**: It only appears while Roblox is running and not minimized. Press F4 if you hid it
- **Hotkeys not working**: Another app may own the key. Rebind in Settings › Hotkeys
- **Fish detection failing**: Open Setup › Fishing bar area. If the match score is low, raise Settings › Color tolerance or redraw the area tighter around the bar
- **Devil fruit not detected**: Setup › Drop message area › Read now shows exactly what the OCR sees. On Linux, confirm `tesseract-data-eng` is installed
- **Fruit spawns not detected**: Ensure the drop message area covers the spawn popup
- **Auto-purchase failing**: Verify the Confirm and Quantity points are set and you are standing next to the bait barrel
- **Logs**: Settings › Data folder › `logs/`

### Devil Fruit Issues

- **Fruits not being stored**: Check if OCR detected the fruit in the Dashboard activity feed
- **Storage sequence running without fruit**: Ensure the drop message area only covers the popup
- **Wrong inventory slot**: Verify the fruit keys match your hotbar
- **Rod not switching back**: Check the rod key (slot `1`) and bait point configuration

---

## 📁 Project Structure

```
src-tauri/src/
├── core/platform/       # OS traits plus isolated windows/ and linux/ backends
├── core/vision.rs       # Bar / fish / marker detection
├── core/fruit.rs        # Drop and spawn text matching
├── core/controller.rs   # Reel controller
├── bot/                 # State machine, actions, watchdog, session stats
├── config.rs            # Settings, presets, v3 import
├── webhook.rs           # Discord embeds
└── commands.rs          # Tauri command surface
src/
├── windows/             # Hud, Panel, Overlay
├── pages/               # Dashboard, Setup, Features, Settings
└── components/          # Shared UI pieces
```

Everything OS-specific sits behind traits in `src-tauri/src/core/platform/mod.rs`, so other capture or OCR backends can be added without touching the bot logic. Fruit names and drop phrases live in settings under `lexicon`, so a game update does not need a rebuild.

## 🤝 Contributing

This is an open-source project! Feel free to:

- Report bugs and issues
- Suggest new features
- Submit pull requests
- Join our Discord community

**💬 Discord:** https://discord.gg/unPZxXAtfb

## License

MIT. See [LICENSE](LICENSE).

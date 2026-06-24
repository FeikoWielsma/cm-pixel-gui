# cm-pixel-gui — implementation plan

A native desktop companion app for the **Cooler Master MasterLiquid Atmos II Pixel LED**
hexagonal screen. Built with **Tauri v2** (Rust backend + web UI). Runs at startup, lives in
the system tray, and drives the screen with a self-contained native Rust driver — **no Python
runtime**. Sibling to the Python library [`cm-pixel`](https://github.com/FeikoWielsma/cm-pixel);
protocol is identical (see `reference/PROTOCOL.md`).

This document is written so the work can be split across **subagents working in parallel**.
The shared contracts in §4 are the glue — agree on those first; everything else slots in behind them.

---

## 1. Goals / non-goals

**Goals**
- Single self-contained Windows app (.exe + installer), no external runtime.
- System tray with quick controls; optional main window with full UI + live preview.
- Run-at-startup (toggle), single-instance, persisted settings.
- Modes: Off, Solid color, Image, GIF, Text/Scroll, Procedural animation (all 19 from `cm-pixel`).
- Brightness (0–100) and rotation (0/90/180/270), applied host-side (same as protocol).
- Graceful handling when Cooler Master MasterCTRL owns the device (detect + offer to stop service).

**Non-goals (v1)**
- Pump/fan control (those are PWM, not USB).
- Cross-platform polish (target Windows first; keep core OS-agnostic where cheap).
- Brightness/rotation device commands (none exist — host-side only).

---

## 2. Stack & key decisions

| Concern | Choice | Notes |
|---|---|---|
| Shell | **Tauri v2** | tray, autostart, single-instance plugins; WebView2 present on Win11 |
| Backend | **Rust** | HID via `hidapi` crate; render loop in a background thread |
| Frontend | **Svelte 5 + Vite + TypeScript** | small, reactive; fine for a settings UI |
| HID | `hidapi` crate (0.6+) | open interface `usage_page == 0xFF01` |
| Image/GIF | `image` crate | decode + resize to 32×32 |
| Config | `tauri-plugin-store` | JSON in app config dir |
| Autostart | `tauri-plugin-autostart` | registry Run key on Windows |
| Single instance | `tauri-plugin-single-instance` | focus existing window on relaunch |
| Packaging | Tauri bundler (NSIS/MSI) | code signing optional/out of scope v1 |

Tauri CLI is **not installed** — install with `cargo install tauri-cli --locked` (or use
`pnpm dlx @tauri-apps/cli@latest`). Frontend deps via **pnpm** (already installed).

---

## 3. Repository layout (target)

```
cm-pixel-gui/
  PLAN.md                  # this file
  README.md  LICENSE  .gitignore
  reference/               # protocol + Python sources to port from (already present)
  package.json             # frontend (pnpm)
  vite.config.ts  svelte.config.js  tsconfig.json
  index.html
  src/                     # Svelte frontend
    main.ts  App.svelte
    lib/
      api.ts               # typed wrappers over Tauri invoke()/events
      HexPreview.svelte    # live 556-LED hexagon preview canvas
      controls/*.svelte    # mode/color/brightness/animation pickers
  src-tauri/
    Cargo.toml  build.rs  tauri.conf.json
    icons/                 # tray + app icons
    src/
      main.rs              # app bootstrap, plugins, tray, single-instance
      device.rs            # HID driver (T1)
      layout.rs            # embedded 32×32→556 map (T1)
      canvas.rs            # Canvas/Frame type, rotation, brightness (T1)
      anim/
        mod.rs             # Animation trait + registry (T2)
        procedural.rs      # plasma…twinkle (T2)
        rubik.rs           # self-solving cube (T3)
        text.rs            # bitmap font + scroll (T4)
        media.rs           # image/gif (T4)
      engine.rs            # render loop thread + Mode state machine (T5)
      commands.rs          # #[tauri::command] IPC handlers (T5)
      config.rs            # persisted settings (T6)
      tray.rs              # tray icon + menu (T6)
  .github/workflows/       # CI + release installer build (T8)
```

---

## 4. Shared contracts (define & merge FIRST — unblocks parallel work)

These are the interfaces every task codes against. Land them as a first commit (stubs that
compile) so T1–T6 can proceed independently.

### 4.1 Canvas / Frame (`canvas.rs`)
```rust
pub const W: usize = 32;
pub const H: usize = 32;
pub type Rgb = [u8; 3];

#[derive(Clone)]
pub struct Canvas { pub px: [[Rgb; W]; H] }   // row-major, px[y][x]

impl Canvas {
    pub fn black() -> Self;
    pub fn set(&mut self, x: usize, y: usize, c: Rgb);
    pub fn fill(&mut self, c: Rgb);
    pub fn rotated(&self, deg: u16) -> Canvas;          // 0/90/180/270
    pub fn with_brightness(&self, pct: u8) -> Canvas;   // linear scale
}
```

### 4.2 Layout + Device (`layout.rs`, `device.rs`)
```rust
// layout.rs
pub const NLED: usize = 556;
pub static LAYOUT: [u16; 1024];          // include_str! parse of reference/layout.json
pub static CELL_OF_LED: [usize; NLED];   // precomputed inverse (build at init)

// device.rs
pub struct PixelDevice { /* hidapi handle */ }
impl PixelDevice {
    pub fn open() -> anyhow::Result<Self>;             // find usage_page 0xFF01
    pub fn is_present() -> bool;
    /// Encode + write a frame: 28 reports of 65 bytes:
    /// [0x00, 0x80, 0xDD, idx_hi, idx_lo] + 60 data; 556*RGB in CELL_OF_LED order.
    pub fn send(&mut self, frame: &Canvas) -> anyhow::Result<()>;
}
```
**Encoder MUST match `reference/PROTOCOL.md` byte-for-byte** (verifiable against cm-pixel output).

### 4.3 Animation (`anim/mod.rs`)
```rust
pub trait Animation: Send {
    /// Render the next frame. `t` = seconds since start, `dt` = seconds since last call.
    fn render(&mut self, t: f32, dt: f32) -> Canvas;
}
pub struct AnimInfo { pub id: &'static str, pub label: &'static str }
pub fn list() -> &'static [AnimInfo];
pub fn make(id: &str, speed: f32) -> Option<Box<dyn Animation>>;
```
IDs (match cm-pixel): `plasma rainbow swirl ripple breathe sparkle fire starfield tunnel
metaballs matrix fireworks aurora comet pinwheel bounce interference twinkle rubik`.

### 4.4 Engine mode + status (`engine.rs`)
```rust
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Mode {
    Off,
    Solid { rgb: Rgb },
    Image { path: String },
    Gif   { path: String },
    Text  { text: String, rgb: Rgb, scroll: bool },
    Anim  { id: String, speed: f32 },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub mode: Mode, pub brightness: u8, pub rotation: u16,
    pub fps: u16, pub autostart: bool,
}

#[derive(Serialize, Clone)]
pub struct Status { pub device_present: bool, pub running: bool, pub mode: Mode, pub conflict: bool }
```
Engine runs a thread: owns `PixelDevice`, current `Mode`, brightness, rotation, fps. Applies
`rotated()` + `with_brightness()` before `send()`. Receives `Settings` updates over an
`mpsc`/`watch` channel. Emits the rendered frame for the UI preview (throttled, see §4.5).

### 4.5 Tauri IPC (`commands.rs` ⇄ `src/lib/api.ts`)
Commands (Rust `#[tauri::command]`):
```
get_status() -> Status
get_settings() -> Settings
set_mode(mode: Mode)
set_brightness(pct: u8)
set_rotation(deg: u16)
set_fps(v: u16)
set_autostart(on: bool)
list_animations() -> Vec<AnimInfo>
pick_file(kind: "image"|"gif") -> Option<String>     // native dialog
stop_cm_service() -> Result<()>                       // elevated helper (see §7)
```
Events (Rust → JS):
```
"status"   : Status              // on device connect/disconnect, mode change
"preview"  : { px: [[u8;3]; ...] } OR base64 RGB(32x32)   // ~15fps, for HexPreview
```

---

## 5. Work breakdown (tasks for subagents)

Each task lists: **scope · files · crate/deps · depends-on · acceptance**. Tasks marked ‖ can run
in parallel once §4 contracts are merged.

### T0 — Scaffold + contracts  *(do first, single agent)*
- Scope: `pnpm create tauri-app` (Svelte+TS), add plugins, commit §4 contracts as compiling stubs.
- Files: whole skeleton, `tauri.conf.json`, stub modules with the signatures above.
- Acceptance: `cargo tauri dev` opens a blank window; `cargo build` green; stubs compile.

### T1 — Rust HID driver ‖
- Scope: `layout.rs`, `device.rs`, `canvas.rs` (rotation+brightness). Port encoder from
  `reference/PROTOCOL.md` + `python_device.py`.
- Deps: `hidapi`, `anyhow`. Plus a tiny `examples/blink.rs` to test on hardware.
- Acceptance: `cargo run --example blink` fills the screen green; encoder unit test reproduces
  the documented 28×65-byte reports; a solid-red frame yields `FF 00 00…` like cm-pixel.

### T2 — Procedural animations ‖
- Scope: port the 18 procedural effects from `reference/python_animations.py` into
  `anim/procedural.rs` behind the `Animation` trait; register in `anim/mod.rs`.
- Deps: none beyond std (+ maybe `glam`/`rand`). Helper `hsv_to_rgb`.
- Acceptance: each id renders 32×32 frames without panic; visual parity check vs Python GIFs
  (dump to PNG via the example in T1 or a `--dump` mode).

### T3 — Rubik animation ‖
- Scope: port `reference/python_rubik.py` (cube model, layer turns, isometric painter render,
  self-solving) into `anim/rubik.rs`.
- Deps: `glam` (or hand-rolled 3×3 mat) for rotations.
- Acceptance: scramble+inverse returns to solved (unit test, like the Python one); render stays
  inside the hexagon mask; visually correct iso cube.

### T4 — Text + media ‖
- Scope: `anim/text.rs` (port 5px font from `python_font.py`, static + scroll) and
  `anim/media.rs` (load image/gif via `image` crate, resize-cover to 32×32, gif timing).
- Acceptance: text renders legibly; gif plays at correct speed; both expose `Animation` impls.

### T5 — Engine + commands  *(depends: T1; integrates T2–T4)*
- Scope: `engine.rs` render-loop thread + `Mode` state machine + device hotplug/retry; `commands.rs`
  IPC; wire preview-frame events (throttled). Conflict detection (device open fails → `conflict`).
- Acceptance: switching modes/brightness/rotation from a test harness changes the screen live;
  survives unplug/replug; preview events arrive in the webview.

### T6 — Tray, autostart, config, single-instance  *(depends: T0; light dep on T5 for actions)*
- Scope: `tray.rs` (icon + menu: Show/Hide, presets submenu, Start/Stop, Quit; left-click toggles
  window), `config.rs` (load/save `Settings` via store; apply on launch), autostart + single-instance
  plugins, "start hidden/minimized to tray" launch flag.
- Acceptance: closing window hides to tray; relaunch focuses; autostart toggle adds/removes Run key;
  settings persist across restarts; app can boot straight to last mode with no window.

### T7 — Frontend UI  *(depends: T0 contracts; mocks until T5)*
- Scope: `App.svelte` + `lib/api.ts` + components: mode selector, color picker, animation list with
  speed slider, brightness/rotation, image/gif file pickers, device-status banner (+ "Stop MasterCTRL"
  action when conflict), `HexPreview.svelte` rendering the 556-LED hexagon from `preview` events.
- Acceptance: every control invokes the right command; preview mirrors the screen; looks clean.

### T8 — Packaging, CI, release  *(depends: most)*
- Scope: app/tray icons, NSIS installer config, `.github/workflows` (build on push; build+attach
  installer on release), README with screenshots/GIFs.
- Acceptance: `cargo tauri build` produces a working installer; CI green; release has the .exe/.msi.

### T9 — MasterCTRL coexistence helper  *(optional, depends T5)*
- Scope: detect CMAgent/MasterCTRLService running; `stop_cm_service`/`start_cm_service` via elevated
  PowerShell (`Stop-Service`/`Start-Service MasterCTRLService`); surface in UI + tray.
- Acceptance: one click frees/returns the device; clear UX about the exclusive-access tradeoff.

---

## 6. Sequencing / parallelism

1. **T0** (scaffold + contracts) — blocking, one agent.
2. Then in parallel: **T1, T2, T3, T4, T7** (T7 against mocked api).
3. **T5** once T1 lands (pulls in T2–T4 as they finish).
4. **T6** after T0 (actions wired as T5 lands).
5. **T8/T9** last.

Critical path: T0 → T1 → T5 → T6 → T8. Animations (T2/T3/T4) and UI (T7) are wide parallel lanes.

---

## 7. Risks & notes
- **Exclusive device access:** only one process can drive the screen. If MasterCTRL/CMAgent is
  running it owns the HID interface → our `open()` fails. Detect and surface (T5/T9). Clean way to
  free it: `Stop-Service MasterCTRLService` (admin); restore with `Start-Service` (see cm-pixel notes).
- **No init handshake needed:** streaming frames displays immediately (confirmed). On exit, leave the
  screen black (send a black frame) or let the device fall back to its offline mode.
- **Autostart UX:** start minimized to tray (launch arg/config), don't pop the window on boot.
- **hidapi on Windows:** uses the OS HID stack; report id 0 prepended (65-byte writes). No admin needed
  to talk to the device itself (only to stop the CM service).
- **Encoder correctness:** add a unit test asserting bytes vs the documented format; optionally diff
  against a captured frame.
- **Code signing:** unsigned installer triggers SmartScreen; document, defer signing.

## 8. Protocol reference (summary; full in `reference/PROTOCOL.md`)
- Device `2516:021C`, HID, frame interface `usage_page 0xFF01`, ep interrupt OUT.
- Frame = 28×64B reports: `80 DD <pkt:u16 BE> + 60B`; concat → 556×RGB888 in `layout.json` order; pad.
- 32×32 grid, 556 LEDs (hexagon), brightness + rotation are host-side, ~15 fps.

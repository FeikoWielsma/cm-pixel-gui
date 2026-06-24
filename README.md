# cm-pixel-gui

Native desktop companion app for the **Cooler Master MasterLiquid Atmos II Pixel LED**
hexagonal screen — system-tray UI, run-at-startup, live preview. Built with **Tauri v2**
(Rust + web), with a self-contained native Rust driver (no Python runtime).

Sibling to the Python library **[cm-pixel](https://github.com/FeikoWielsma/cm-pixel)**; it
reuses the same reverse-engineered USB protocol (`reference/PROTOCOL.md`).

> **Status: planning / WIP.** The full implementation plan — structured for parallel work —
> is in **[PLAN.md](PLAN.md)**. Reference protocol + the Python sources being ported live in
> [`reference/`](reference/).

## Planned features
- System tray with quick controls; main window with full UI + live hexagon preview
- Modes: Off · Solid color · Image · GIF · Text/Scroll · 19 procedural animations
  (incl. the self-solving isometric Rubik's cube)
- Brightness & rotation (host-side)
- Run-at-startup, single-instance, persisted settings
- Detects when Cooler Master MasterCTRL owns the device and offers to free it

## License
MIT — see [LICENSE](LICENSE). Unofficial; not affiliated with Cooler Master.

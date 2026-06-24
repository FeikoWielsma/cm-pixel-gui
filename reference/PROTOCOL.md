# Atmos V2 "Pixel" — USB protocol (reverse engineered)

Cooler Master MasterLiquid Atmos II Pixel LED — hexagonal pixel screen.
Status: **frame format fully decoded & verified** against ground-truth images (2026-06-24).

## Device
- USB `VID 0x2516`, `PID 0x021C`, name "Atmos V2 - Pixel". HID composite, 3 interfaces:
  - **iface 0** `UsagePage 0xffff` — command/keepalive channel (control SET_REPORT)
  - **iface 1** `UsagePage 0xff01` — **frame channel** (interrupt OUT, ep 0x02)
  - iface 2 — dummy HID mouse, ignore
- Panel: 32×32 addressing grid, **556 real LEDs** in a hexagon (see `re/lamp_layout_32x32.json`).

## Frame channel (iface 1, interrupt OUT endpoint 0x02)
A full frame = **28 HID output reports**, each **64 bytes on the wire**:

```
byte 0     0x80                 (magic)
byte 1     0xDD                 (magic)
byte 2..3  packet index, u16 BE (0x0000 .. 0x001B = 0..27)
byte 4..63 60 bytes of pixel payload
```

- Concatenate the 28 × 60 = **1680 payload bytes**.
- First **1668 bytes = 556 LEDs × RGB888**, channel order **R,G,B**. Last 12 bytes = padding (0).
- LED order = the `lamp-layout` index order (1..556), i.e. the serpentine wiring.
  Map a 32×32 image to the stream: for grid cell `(x,y)`, `lamp = layout[y*32+x]`;
  if `lamp != 0`, that LED's bytes go at offset `(lamp-1)*3`.
- Verified: red img → `(255,0,0)` solids; xy-encoded img reproduces grid coords in index
  order; corner-probe renders red=top, green=left, blue=right, yellow=bottom (no rotation/flip).
- Streaming is continuous (~8–15 fps) in "ssr" mode; resending the same frame is fine.

### Writing with hidapi
iface 1 OutputReportByteLength = 65 (1 report-id byte + 64). Report id is 0, so prepend `0x00`:
```
buf = bytes([0x00, 0x80, 0xDD, idx>>8, idx&0xFF]) + payload60   # 65 bytes
dev.write(buf)   # x28, idx 0..27
```

## Command channel (iface 0, control SET_REPORT, bmRequestType=0x21 bRequest=0x09)
Steady-state keepalives seen (every ~0.1–0.5 s), 2-byte Output reports:
- report id `0x02`, data `02 03`
- report id `0x16`, data `16 01`
GET_REPORT (0xA1/0x01) status reads also occur (TLV blob, report id ~0x14).

### Init / mode-set
**CONFIRMED (live test):** streaming frames to iface 1 displays them immediately — **no init
or iface-0 keepalive is required.** When the CM software exits, the device drops to an
autonomous "offline/screensaver" mode; the first streamed frame takes over the panel.

**Brightness & rotation are HOST-SIDE (confirmed by capture 160817):** dragging the brightness
slider scales the streamed RGB values (max 232→185→122→81→32→…→232) with no distinct wire
command; rotation likewise just re-orients the pixel data before streaming. So there is **no
device brightness/rotation command** — both are implemented in software (`cmpixel` does this:
`np.rot90` + linear RGB scale). The hexagon is not 90°-symmetric, so 90/270 clip at the vertices
(same as the official app); 180° is exact.

Steady-state command channel (iface 0 SET_REPORT) only carries keepalives: report id 0x02 `02 03`,
id 0x16 `16 01`, plus occasional id 0x15 `15 00/15 01` (UI/state ping). None needed to display.

## Files
- `re/lamp_layout_32x32.json` — 1024 ints, 0 = no LED, else 1..556 LED index (row-major, w=32)
- `re/pixel_caps.json` — full device capability descriptor (from CMAgent.exe)
- `re/decode_frames.py` — capture → reconstructed PNGs
- `re/capture/frames/` — reconstructed ground-truth frames (verification)

"""Driver for the Cooler Master Atmos V2 'Pixel' hexagonal LED screen.

Protocol fully documented in PROTOCOL.md. Screen is a 32x32 grid with 556 real
LEDs in a hexagon; frames are streamed over HID interface 1 as 28 x 64-byte reports.
"""
import json
import os
import time

import hid

VID = 0x2516
PID = 0x021C
FRAME_USAGE_PAGE = 0xFF01   # interface 1 = frame channel
GRID_W = GRID_H = 32
NLED = 556
PACKETS = 28
PAYLOAD = 60                # data bytes per report
MAGIC = (0x80, 0xDD)

_LAYOUT = json.load(open(os.path.join(os.path.dirname(__file__), "layout.json")))


class PixelDisplay:
    """Connect to the screen and push 32x32 RGB frames."""

    def __init__(self, layout=_LAYOUT, brightness=100, rotation=0):
        self.layout = layout
        self.dev = None
        self.brightness = brightness   # 0..100, host-side scale (device has no brightness cmd)
        self.rotation = rotation        # 0/90/180/270, host-side rotate
        # precompute: led_index(0-based) -> grid cell (for fast image->stream)
        self.cell_of_led = [0] * NLED
        for cell, lamp in enumerate(layout):
            if lamp:
                self.cell_of_led[lamp - 1] = cell

    # ---- connection ----
    def open(self):
        path = None
        for d in hid.enumerate(VID, PID):
            if d["usage_page"] == FRAME_USAGE_PAGE:
                path = d["path"]
                break
        if path is None:
            raise RuntimeError(
                "Frame interface (usage_page 0xff01) not found. "
                "Is the cooler connected and the CM software closed?"
            )
        self.dev = hid.device()
        self.dev.open_path(path)
        return self

    def close(self):
        if self.dev:
            self.dev.close()
            self.dev = None

    def __enter__(self):
        return self.open()

    def __exit__(self, *exc):
        self.close()

    # ---- frame building ----
    def _stream_from_grid(self, pixels):
        """pixels: list of 1024 (r,g,b). Returns 1668-byte LED stream in index order."""
        buf = bytearray(NLED * 3)
        for led, cell in enumerate(self.cell_of_led):
            r, g, b = pixels[cell]
            o = led * 3
            buf[o], buf[o + 1], buf[o + 2] = r & 0xFF, g & 0xFF, b & 0xFF
        return bytes(buf)

    def send_stream(self, led_stream):
        """led_stream: 1668 bytes (556*RGB) in LED index order -> write 28 reports."""
        if self.dev is None:
            raise RuntimeError("not open")
        data = bytes(led_stream).ljust(PACKETS * PAYLOAD, b"\x00")  # pad to 1680
        for i in range(PACKETS):
            chunk = data[i * PAYLOAD:(i + 1) * PAYLOAD]
            report = bytes([0x00, MAGIC[0], MAGIC[1], (i >> 8) & 0xFF, i & 0xFF]) + chunk
            self.dev.write(report)

    def send_grid(self, pixels):
        """pixels: list of 1024 (r,g,b) tuples (row-major, 32 wide)."""
        self.send_stream(self._stream_from_grid(pixels))

    def send_array(self, arr):
        """arr: numpy (32, 32, 3) uint8 (row-major). Applies rotation+brightness."""
        import numpy as np
        a = np.asarray(arr, dtype=np.uint8)
        if self.rotation % 360:
            a = np.rot90(a, k=(self.rotation // 90) % 4)
        if self.brightness != 100:
            a = (a.astype(np.uint16) * max(0, min(100, self.brightness)) // 100).astype(np.uint8)
        stream = a.reshape(-1, 3)[self.cell_of_led]   # (556, 3) in LED index order
        self.send_stream(stream.tobytes())

    def send_canvas(self, canvas):
        """canvas: cmpixel.canvas.Canvas"""
        self.send_array(canvas.buf)

    def send_image(self, img):
        """img: PIL Image; resized to 32x32 RGB and pushed."""
        from PIL import Image
        img = img.convert("RGB").resize((GRID_W, GRID_H), Image.NEAREST)
        self.send_grid(list(img.getdata()))

    def fill(self, rgb):
        self.send_grid([tuple(rgb)] * (GRID_W * GRID_H))


def main():
    import sys
    color = (0, 80, 0)
    with PixelDisplay() as d:
        print("connected. filling green; Ctrl+C to stop.")
        try:
            while True:
                d.fill(color)
                time.sleep(0.1)   # keep streaming ~10fps
        except KeyboardInterrupt:
            d.fill((0, 0, 0))
            print("\ncleared.")


if __name__ == "__main__":
    main()

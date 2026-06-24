"""Procedural animations. Each is a generator yielding (32,32,3) uint8 frames.

All take a `speed` multiplier; most accept extra knobs. They run forever.
Frames are full-grid; the hexagon mask is applied by the player/device anyway.
"""
import numpy as np

from .canvas import GRID_W, GRID_H, MASK
from .rubik import rubik

_Y, _X = np.mgrid[0:GRID_H, 0:GRID_W].astype(np.float32)
_CX, _CY = (GRID_W - 1) / 2.0, (GRID_H - 1) / 2.0
_DX, _DY = _X - _CX, _Y - _CY
_R = np.sqrt(_DX**2 + _DY**2)
_ANG = np.arctan2(_DY, _DX)


def hsv_to_rgb(h, s, v):
    """h,s,v in [0,1] arrays -> (H,W,3) uint8."""
    h = (h % 1.0) * 6.0
    i = np.floor(h).astype(int)
    f = h - i
    p = v * (1 - s)
    q = v * (1 - f * s)
    t = v * (1 - (1 - f) * s)
    i = i % 6
    r = np.choose(i, [v, q, p, p, t, v])
    g = np.choose(i, [t, v, v, q, p, p])
    b = np.choose(i, [p, p, t, v, v, q])
    return (np.stack([r, g, b], axis=-1).clip(0, 1) * 255).astype(np.uint8)


def plasma(speed=1.0):
    t = 0.0
    while True:
        v = (np.sin(_X / 4.0 + t)
             + np.sin(_Y / 3.0 - t)
             + np.sin((_X + _Y) / 4.0 + t)
             + np.sin(_R / 3.0 - t))
        h = (v + 4) / 8.0
        yield hsv_to_rgb(h, np.ones_like(h), np.ones_like(h))
        t += 0.15 * speed


def rainbow(speed=1.0, vertical=False):
    t = 0.0
    base = _Y if vertical else _X
    while True:
        h = base / GRID_W + t
        yield hsv_to_rgb(h, np.ones_like(h), np.ones_like(h))
        t += 0.02 * speed


def swirl(speed=1.0):
    t = 0.0
    while True:
        h = (_ANG / (2 * np.pi)) + _R / 16.0 + t
        yield hsv_to_rgb(h, np.ones_like(h), np.ones_like(h))
        t += 0.05 * speed


def ripple(speed=1.0, color=None):
    t = 0.0
    while True:
        wave = (np.sin(_R - t) + 1) / 2.0
        if color is None:
            out = hsv_to_rgb((_R / 16.0 - t / 6.0), np.ones_like(wave), wave)
        else:
            out = (np.array(color, np.float32) * wave[..., None]).astype(np.uint8)
        yield out
        t += 0.3 * speed


def breathe(color=(0, 120, 255), speed=1.0):
    t = 0.0
    col = np.array(color, np.float32)
    while True:
        v = (np.sin(t) + 1) / 2.0
        frame = np.empty((GRID_H, GRID_W, 3), np.uint8)
        frame[:] = (col * v).astype(np.uint8)
        yield frame
        t += 0.08 * speed


def sparkle(color=(255, 255, 255), speed=1.0, fade=0.85, rate=6):
    buf = np.zeros((GRID_H, GRID_W, 3), np.float32)
    leds = np.argwhere(MASK)
    while True:
        buf *= fade
        for _ in range(int(rate * speed)):
            y, x = leds[np.random.randint(len(leds))]
            buf[y, x] = color
        yield buf.clip(0, 255).astype(np.uint8)


def fire(speed=1.0):
    heat = np.zeros((GRID_H + 2, GRID_W), np.float32)
    # fire palette: black -> red -> orange -> yellow -> white
    stops = np.array([[0, 0, 0], [80, 0, 0], [220, 50, 0],
                      [255, 160, 0], [255, 255, 120]], np.float32)
    xp = np.linspace(0, 1, len(stops))
    grad = np.stack([np.interp(np.linspace(0, 1, 256), xp, stops[:, c]) for c in range(3)], axis=-1)
    while True:
        heat[-1] = np.random.rand(GRID_W) * 255  # spark row at bottom
        # cool + rise
        nh = (heat[1:] * 0.5 + heat[:-1] * 0.0)
        rolled = (np.roll(heat, -1, axis=0) * 0.42 + heat * 0.30
                  + np.roll(heat, (-1, 1), (0, 1)) * 0.14
                  + np.roll(heat, (-1, -1), (0, 1)) * 0.14)
        heat[:] = rolled.clip(0, 255)
        idx = heat[:GRID_H].astype(int).clip(0, 255)
        yield grad[idx].astype(np.uint8)
        if speed != 1.0:
            pass


def starfield(speed=1.0, density=0.04):
    stars = []  # (x, y, z)
    while True:
        if np.random.rand() < density * 8:
            stars.append([np.random.uniform(-_CX, _CX), np.random.uniform(-_CY, _CY), 16.0])
        frame = np.zeros((GRID_H, GRID_W, 3), np.uint8)
        alive = []
        for s in stars:
            s[2] -= 0.4 * speed
            if s[2] <= 1:
                continue
            px = int(_CX + s[0] * 8 / s[2])
            py = int(_CY + s[1] * 8 / s[2])
            if 0 <= px < GRID_W and 0 <= py < GRID_H:
                b = int(255 * (1 - s[2] / 16))
                frame[py, px] = (b, b, b)
                alive.append(s)
        stars[:] = alive[-200:]
        yield frame


def _solid(hue, sat, val):
    """hsv_to_rgb for scalar h,s,v -> (3,) float array."""
    return hsv_to_rgb(np.array([[hue]]), np.array([[sat]]), np.array([[val]]))[0, 0].astype(float)


def tunnel(speed=1.0):
    """Hypnotic radial tunnel rushing inward."""
    t = 0.0
    v = 1.0 / (_R + 0.6)
    u = _ANG / (2 * np.pi)
    vig = 1 - np.clip(_R / (GRID_W * 0.6), 0, 1)
    while True:
        stripes = 0.5 + 0.5 * np.sin(v * 18 - t * 4)
        hue = (v * 1.5 + u + t * 0.05) % 1.0
        val = stripes * (0.3 + 0.7 * vig)
        yield hsv_to_rgb(hue, np.ones_like(hue), val)
        t += 0.05 * speed


def metaballs(speed=1.0, n=4):
    """Smooth gooey blobs drifting and merging."""
    t = 0.0
    ph = np.random.rand(n, 2) * 2 * np.pi
    while True:
        field = np.zeros((GRID_H, GRID_W), np.float32)
        for i in range(n):
            cx = GRID_W / 2 + GRID_W * 0.34 * np.sin(t * 0.6 + ph[i, 0])
            cy = GRID_H / 2 + GRID_H * 0.34 * np.cos(t * 0.5 + ph[i, 1])
            field += 30.0 / ((_X - cx) ** 2 + (_Y - cy) ** 2 + 8)
        hue = (field * 0.3 + t * 0.05) % 1.0
        val = np.clip(field * 0.9, 0, 1)
        yield hsv_to_rgb(hue, np.ones_like(hue), val)
        t += 0.05 * speed


def matrix(speed=1.0, fade=0.78):
    """Falling green 'digital rain' columns."""
    buf = np.zeros((GRID_H, GRID_W, 3), np.float32)
    heads = np.zeros(GRID_W)
    spd = 0.3 + np.random.rand(GRID_W) * 0.7
    active = np.random.rand(GRID_W) < 0.4
    while True:
        buf *= fade
        for x in range(GRID_W):
            if not active[x]:
                if np.random.rand() < 0.03:
                    active[x] = True
                    heads[x] = 0
                continue
            y = int(heads[x])
            if 0 <= y < GRID_H:
                buf[y, x] = (170, 255, 170)
            heads[x] += spd[x] * speed
            if heads[x] > GRID_H + 4:
                active[x] = False
        yield buf.clip(0, 255).astype(np.uint8)


def fireworks(speed=1.0, gravity=0.05):
    """Launch bursts that explode and fall."""
    parts = []
    buf = np.zeros((GRID_H, GRID_W, 3), np.float32)
    while True:
        buf *= 0.82
        if np.random.rand() < 0.06 * speed or not parts:
            cx, cy = np.random.uniform(8, 24), np.random.uniform(6, 18)
            col = _solid(np.random.rand(), 1.0, 1.0)
            n = np.random.randint(18, 30)
            for k in range(n):
                a = 2 * np.pi * k / n
                sp = np.random.uniform(0.4, 1.2)
                parts.append([cx, cy, np.cos(a) * sp, np.sin(a) * sp, 1.0, col])
        alive = []
        for p in parts:
            p[0] += p[2] * speed
            p[1] += p[3] * speed
            p[3] += gravity * speed
            p[4] -= 0.02 * speed
            if p[4] <= 0:
                continue
            xi, yi = int(round(p[0])), int(round(p[1]))
            if 0 <= xi < GRID_W and 0 <= yi < GRID_H:
                buf[yi, xi] = np.array(p[5]) * p[4]
            alive.append(p)
        parts[:] = alive
        yield buf.clip(0, 255).astype(np.uint8)


def aurora(speed=1.0):
    """Northern-lights curtains, green through purple."""
    t = 0.0
    while True:
        center = GRID_H * 0.5 + 6 * np.sin(_X * 0.3 + t) + 3 * np.sin(_X * 0.7 - t * 1.3)
        val = np.clip(np.exp(-((_Y - center) ** 2) / 18.0), 0, 1)
        hue = (0.33 + 0.18 * np.sin(_X * 0.2 + t * 0.5) + _Y * 0.012) % 1.0
        yield hsv_to_rgb(hue, np.full_like(hue, 0.85), val)
        t += 0.06 * speed


def comet(speed=1.0):
    """A color-cycling comet orbiting with a fading tail."""
    buf = np.zeros((GRID_H, GRID_W, 3), np.float32)
    t = 0.0
    while True:
        buf *= 0.80
        r = GRID_W * 0.32
        x = GRID_W / 2 + r * np.cos(t)
        y = GRID_H / 2 + r * np.sin(t)
        xi, yi = int(round(x)), int(round(y))
        if 0 <= xi < GRID_W and 0 <= yi < GRID_H:
            buf[yi, xi] = _solid((t * 0.1) % 1, 1.0, 1.0)
        yield buf.clip(0, 255).astype(np.uint8)
        t += 0.15 * speed


def pinwheel(speed=1.0, arms=5):
    """Rotating rainbow pinwheel."""
    t = 0.0
    while True:
        val = 0.5 + 0.5 * np.sin(_ANG * arms + _R * 0.5 - t * 3)
        hue = (_ANG / (2 * np.pi) + t * 0.1) % 1.0
        yield hsv_to_rgb(hue, np.ones_like(hue), val ** 1.5)
        t += 0.05 * speed


def bounce(speed=1.0):
    """A bouncing ball with a motion trail."""
    buf = np.zeros((GRID_H, GRID_W, 3), np.float32)
    pos = np.array([16.0, 16.0])
    vel = np.array([0.7, 0.5])
    lo, hi = 4, 27
    t = 0.0
    while True:
        buf *= 0.75
        pos += vel * speed
        for k in range(2):
            if pos[k] < lo:
                pos[k], vel[k] = lo, -vel[k]
            if pos[k] > hi:
                pos[k], vel[k] = hi, -vel[k]
        buf[int(round(pos[1])), int(round(pos[0]))] = _solid((t * 0.02) % 1, 1.0, 1.0)
        yield buf.clip(0, 255).astype(np.uint8)
        t += speed


def interference(speed=1.0):
    """Two moving wave sources rippling and interfering."""
    t = 0.0
    while True:
        c1x, c1y = GRID_W / 2 + 10 * np.sin(t * 0.7), GRID_H / 2 + 10 * np.cos(t * 0.5)
        c2x, c2y = GRID_W / 2 + 10 * np.sin(t * 0.4 + 2), GRID_H / 2 + 10 * np.cos(t * 0.6 + 1)
        d1 = np.sqrt((_X - c1x) ** 2 + (_Y - c1y) ** 2)
        d2 = np.sqrt((_X - c2x) ** 2 + (_Y - c2y) ** 2)
        val = (np.sin(d1 - t * 3) + np.sin(d2 - t * 3)) / 2
        hue = (val * 0.5 + 0.5 + t * 0.03) % 1.0
        yield hsv_to_rgb(hue, np.ones_like(hue), val * 0.5 + 0.5)
        t += 0.05 * speed


def twinkle(speed=1.0, n=45):
    """Colorful stars fading in and out."""
    leds = np.argwhere(MASK)
    stars = []
    while True:
        if len(stars) < n and np.random.rand() < 0.6:
            y, x = leds[np.random.randint(len(leds))]
            stars.append([int(x), int(y), np.random.rand(), 0.0, 0.03 + np.random.rand() * 0.06])
        frame = np.zeros((GRID_H, GRID_W, 3), np.uint8)
        alive = []
        for s in stars:
            s[3] += s[4] * speed
            if s[3] >= np.pi:
                continue
            frame[s[1], s[0]] = _solid(s[2], 0.6, np.sin(s[3]))
            alive.append(s)
        stars[:] = alive
        yield frame


ANIMATIONS = {
    "plasma": plasma,
    "rainbow": rainbow,
    "swirl": swirl,
    "ripple": ripple,
    "breathe": breathe,
    "sparkle": sparkle,
    "fire": fire,
    "starfield": starfield,
    "tunnel": tunnel,
    "metaballs": metaballs,
    "matrix": matrix,
    "fireworks": fireworks,
    "aurora": aurora,
    "comet": comet,
    "pinwheel": pinwheel,
    "bounce": bounce,
    "interference": interference,
    "twinkle": twinkle,
    "rubik": rubik,
}

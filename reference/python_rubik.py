"""Self-solving isometric Rubik's cube — its hexagonal silhouette fits the panel.

A minimal 3x3 cube model with real layer turns, rendered with an isometric
projection + painter's algorithm. It scrambles, then solves itself, forever.
"""
import numpy as np

from .canvas import GRID_W, GRID_H

# sticker color by the cubie face's outward normal (x, y, z)
_COLORS = {
    (1, 0, 0): (0, 210, 40),     # +x green
    (-1, 0, 0): (0, 90, 255),    # -x blue
    (0, 1, 0): (245, 245, 245),  # +y white (top)
    (0, -1, 0): (255, 210, 0),   # -y yellow
    (0, 0, 1): (235, 0, 0),      # +z red
    (0, 0, -1): (255, 95, 0),    # -z orange
}
_LIGHT = np.array([0.35, 1.0, 0.55])
_LIGHT /= np.linalg.norm(_LIGHT)


def _rot(axis, deg):
    r = np.radians(deg)
    c, s = np.cos(r), np.sin(r)
    if axis == 0:
        return np.array([[1, 0, 0], [0, c, -s], [0, s, c]])
    if axis == 1:
        return np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]])
    return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]])


# isometric view: corner-on so we see three faces (silhouette = hexagon)
_VIEW = _rot(0, 35.264) @ _rot(1, 45)
_SCALE = 5.6
_CX, _CY = (GRID_W - 1) / 2.0, (GRID_H - 1) / 2.0


class _Cube:
    def __init__(self):
        self.cubies = []  # each: [pos(int3), [[normal(int3), color], ...]]
        for x in (-1, 0, 1):
            for y in (-1, 0, 1):
                for z in (-1, 0, 1):
                    if x == y == z == 0:
                        continue
                    pos = np.array([x, y, z])
                    st = []
                    for ax, n in ((0, x), (1, y), (2, z)):
                        if n:
                            nrm = np.zeros(3, int)
                            nrm[ax] = n
                            st.append([nrm, _COLORS[tuple(nrm)]])
                    self.cubies.append([pos, st])

    def turn(self, axis, layer, direction):
        R = np.rint(_rot(axis, 90 * direction)).astype(int)
        for cub in self.cubies:
            if cub[0][axis] == layer:
                cub[0] = R @ cub[0]
                for s in cub[1]:
                    s[0] = R @ s[0]


def _basis(normal):
    """two in-plane unit axes for a face normal."""
    ax = int(np.argmax(np.abs(normal)))
    u = np.zeros(3); u[(ax + 1) % 3] = 1
    v = np.zeros(3); v[(ax + 2) % 3] = 1
    return u, v


def _inside(poly, px, py):
    sign = 0
    n = len(poly)
    for i in range(n):
        ax, ay = poly[i]
        bx, by = poly[(i + 1) % n]
        cross = (bx - ax) * (py - ay) - (by - ay) * (px - ax)
        if cross != 0:
            s = 1 if cross > 0 else -1
            if sign == 0:
                sign = s
            elif s != sign:
                return False
    return True


def _fill(buf, pts, color):
    xs = [p[0] for p in pts]; ys = [p[1] for p in pts]
    x0 = max(0, int(np.floor(min(xs)))); x1 = min(GRID_W - 1, int(np.ceil(max(xs))))
    y0 = max(0, int(np.floor(min(ys)))); y1 = min(GRID_H - 1, int(np.ceil(max(ys))))
    for yy in range(y0, y1 + 1):
        for xx in range(x0, x1 + 1):
            if _inside(pts, xx + 0.5, yy + 0.5):
                buf[yy, xx] = color


def _render(cube, moving_axis=None, moving_layer=None, anim_R=None):
    buf = np.zeros((GRID_H, GRID_W, 3), np.uint8)
    quads = []
    for pos, stickers in cube.cubies:
        in_layer = moving_axis is not None and pos[moving_axis] == moving_layer
        for normal, color in stickers:
            n = normal.astype(float)
            center = pos.astype(float) + 0.5 * n
            u, v = _basis(normal)
            corners = [center + u * 0.5 + v * 0.5, center + u * 0.5 - v * 0.5,
                       center - u * 0.5 - v * 0.5, center - u * 0.5 + v * 0.5]
            face_corners = [center + u * 0.42 + v * 0.42, center + u * 0.42 - v * 0.42,
                            center - u * 0.42 - v * 0.42, center - u * 0.42 + v * 0.42]
            if in_layer and anim_R is not None:
                n = anim_R @ n
                corners = [anim_R @ c for c in corners]
                face_corners = [anim_R @ c for c in face_corners]
            nv = _VIEW @ n
            if nv[2] <= 0.02:           # back-face cull
                continue
            shade = 0.45 + 0.55 * max(0.0, float(np.dot(n / np.linalg.norm(n), _LIGHT)))
            depth = 0.0
            black_pts, col_pts = [], []
            for c in corners:
                cv = _VIEW @ c
                depth += cv[2]
                black_pts.append((_CX + cv[0] * _SCALE, _CY - cv[1] * _SCALE))
            for c in face_corners:
                cv = _VIEW @ c
                col_pts.append((_CX + cv[0] * _SCALE, _CY - cv[1] * _SCALE))
            col = tuple(int(c * shade) for c in color)
            quads.append((depth, black_pts, col_pts, col))
    quads.sort(key=lambda q: q[0])     # far first
    for _, black_pts, col_pts, col in quads:
        _fill(buf, black_pts, (8, 8, 8))   # black cube frame
        _fill(buf, col_pts, col)           # colored sticker
    return buf


_AXES = (0, 1, 2)


def _scramble(n):
    moves, last = [], None
    for _ in range(n):
        while True:
            ax = _AXES[np.random.randint(3)]
            layer = (-1, 1)[np.random.randint(2)]
            if (ax, layer) != last:
                break
        moves.append((ax, layer, (-1, 1)[np.random.randint(2)]))
        last = (ax, layer)
    return moves


def rubik(speed=1.0, scramble_len=16):
    """Self-solving isometric Rubik's cube."""
    cube = _Cube()
    queue = []          # list of (move, frames_per_move)
    pause = 0
    while True:
        if pause > 0:
            pause -= 1
            yield _render(cube)
            continue
        if not queue:
            scr = _scramble(scramble_len)
            sol = [(a, l, -d) for (a, l, d) in reversed(scr)]
            queue = [(m, 3) for m in scr] + [(m, 9) for m in sol]
        move, fpm = queue.pop(0)
        ax, layer, direction = move
        steps = max(1, int(round(fpm / max(0.25, speed))))
        for f in range(1, steps + 1):
            theta = 90 * direction * f / steps
            yield _render(cube, ax, layer, _rot(ax, theta))
        cube.turn(ax, layer, direction)
        if not queue:                 # just finished solving
            pause = int(18 / max(0.25, speed))
            yield _render(cube)

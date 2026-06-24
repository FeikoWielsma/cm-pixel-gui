"""Tiny 5-pixel-tall variable-width bitmap font + text rendering helpers.

Glyphs are written as ASCII art ('#' = on) so they're trivial to read/extend.
Lowercase falls back to uppercase; unknown chars render as a blank box.
"""
import numpy as np

from .canvas import Canvas, GRID_W, GRID_H

FONT_H = 5

_GLYPHS = {
    " ": ["..", "..", "..", "..", ".."],
    "0": ["###", "#.#", "#.#", "#.#", "###"],
    "1": [".#.", "##.", ".#.", ".#.", "###"],
    "2": ["###", "..#", "###", "#..", "###"],
    "3": ["###", "..#", ".##", "..#", "###"],
    "4": ["#.#", "#.#", "###", "..#", "..#"],
    "5": ["###", "#..", "###", "..#", "###"],
    "6": ["###", "#..", "###", "#.#", "###"],
    "7": ["###", "..#", "..#", ".#.", ".#."],
    "8": ["###", "#.#", "###", "#.#", "###"],
    "9": ["###", "#.#", "###", "..#", "###"],
    "A": [".#.", "#.#", "###", "#.#", "#.#"],
    "B": ["##.", "#.#", "##.", "#.#", "##."],
    "C": [".##", "#..", "#..", "#..", ".##"],
    "D": ["##.", "#.#", "#.#", "#.#", "##."],
    "E": ["###", "#..", "##.", "#..", "###"],
    "F": ["###", "#..", "##.", "#..", "#.."],
    "G": [".##", "#..", "#.#", "#.#", ".##"],
    "H": ["#.#", "#.#", "###", "#.#", "#.#"],
    "I": ["###", ".#.", ".#.", ".#.", "###"],
    "J": ["..#", "..#", "..#", "#.#", "##."],
    "K": ["#..#", "#.#.", "##..", "#.#.", "#..#"],
    "L": ["#..", "#..", "#..", "#..", "###"],
    "M": ["#...#", "##.##", "#.#.#", "#...#", "#...#"],
    "N": ["#..#", "##.#", "#.##", "#..#", "#..#"],
    "O": [".#.", "#.#", "#.#", "#.#", ".#."],
    "P": ["##.", "#.#", "##.", "#..", "#.."],
    "Q": [".#.", "#.#", "#.#", "#.#", ".##"],
    "R": ["##.", "#.#", "##.", "#.#", "#.#"],
    "S": [".##", "#..", ".#.", "..#", "##."],
    "T": ["###", ".#.", ".#.", ".#.", ".#."],
    "U": ["#.#", "#.#", "#.#", "#.#", "###"],
    "V": ["#.#", "#.#", "#.#", "#.#", ".#."],
    "W": ["#...#", "#...#", "#.#.#", "##.##", "#...#"],
    "X": ["#.#", "#.#", ".#.", "#.#", "#.#"],
    "Y": ["#.#", "#.#", ".#.", ".#.", ".#."],
    "Z": ["###", "..#", ".#.", "#..", "###"],
    ".": [".", ".", ".", ".", "#"],
    ",": [".", ".", ".", "#", "#"],
    ":": [".", "#", ".", "#", "."],
    ";": [".", "#", ".", "#", "#"],
    "-": ["...", "...", "###", "...", "..."],
    "_": ["...", "...", "...", "...", "###"],
    "+": ["...", ".#.", "###", ".#.", "..."],
    "=": ["...", "###", "...", "###", "..."],
    "/": ["..#", "..#", ".#.", "#..", "#.."],
    "\\": ["#..", "#..", ".#.", "..#", "..#"],
    "!": ["#", "#", "#", ".", "#"],
    "?": ["###", "..#", ".##", "...", ".#."],
    "'": ["#", "#", ".", ".", "."],
    "\"": ["#.#", "#.#", "...", "...", "..."],
    "(": [".#", "#.", "#.", "#.", ".#"],
    ")": ["#.", ".#", ".#", ".#", "#."],
    "*": ["#.#", ".#.", "###", ".#.", "#.#"],
    "%": ["#.#", "..#", ".#.", "#..", "#.#"],
    "#": [".#.#.", "#####", ".#.#.", "#####", ".#.#."],
    "<": ["..#", ".#.", "#..", ".#.", "..#"],
    ">": ["#..", ".#.", "..#", ".#.", "#.."],
    "°": ["###", "#.#", "###", "...", "..."],
    "@": ["###", "#.#", "#.#", "#..", "###"],
}
_UNKNOWN = ["###", "#.#", "#.#", "#.#", "###"]


def _glyph(ch):
    return _GLYPHS.get(ch) or _GLYPHS.get(ch.upper()) or _UNKNOWN


def text_width(s, spacing=1):
    if not s:
        return 0
    return sum(len(_glyph(c)[0]) for c in s) + spacing * (len(s) - 1)


def render_text(s, color=(255, 255, 255), bg=(0, 0, 0), spacing=1):
    """Return an (5, width, 3) uint8 array of rendered text."""
    w = max(1, text_width(s, spacing))
    out = np.zeros((FONT_H, w, 3), dtype=np.uint8)
    out[:] = bg
    x = 0
    for c in s:
        g = _glyph(c)
        gw = len(g[0])
        for row in range(FONT_H):
            for col in range(gw):
                if g[row][col] == "#":
                    out[row, x + col] = color
        x += gw + spacing
    return out


def draw_text(canvas, s, x=0, y=None, color=(255, 255, 255), spacing=1):
    """Draw text onto a Canvas. y defaults to vertically centered."""
    img = render_text(s, color=color, spacing=spacing)
    if y is None:
        y = (GRID_H - FONT_H) // 2
    # paste only the 'on' pixels (transparent bg)
    h, w = img.shape[:2]
    for ry in range(h):
        for rx in range(w):
            if img[ry, rx].any():
                canvas.set_pixel(x + rx, y + ry, tuple(int(v) for v in img[ry, rx]))
    return canvas


def scroll_text(s, color=(255, 255, 255), y=None, gap=GRID_W, spacing=1):
    """Yield Canvas frames scrolling `s` right-to-left. Loops forever."""
    img = render_text(s, color=color, spacing=spacing)
    h, w = img.shape[:2]
    if y is None:
        y = (GRID_H - FONT_H) // 2
    total = w + gap
    pos = GRID_W
    while True:
        c = Canvas()
        c.paste(img, x=pos, y=y)
        yield c
        pos -= 1
        if pos < -w:
            pos = gap

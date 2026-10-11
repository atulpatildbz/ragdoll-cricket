# Render the Android launcher icon from the batsman sprites into
# assets/android/res (legacy square PNGs plus an adaptive icon for 8.0+).
# Usage: uv run --with pillow scripts/make-icon.py
from pathlib import Path

from PIL import Image, ImageDraw

root = Path(__file__).resolve().parent.parent
sprites = root / "assets" / "batsman"
res = root / "assets" / "android" / "res"

S = 1024  # working size; an adaptive icon is 108dp, of which the middle 72dp shows


def lerp(a, b, t):
    return tuple(round(x + (y - x) * t) for x, y in zip(a, b))


def hex_rgb(h):
    return ((h >> 16) & 255, (h >> 8) & 255, h & 255)


def background():
    # The game's sky gradient over a strip of outfield.
    img = Image.new("RGBA", (S, S))
    d = ImageDraw.Draw(img)
    top, bottom = hex_rgb(0x3A78E0), hex_rgb(0xC7E3FF)
    grass_y = int(S * 0.74)
    for y in range(grass_y):
        d.line([(0, y), (S, y)], fill=lerp(top, bottom, y / grass_y))
    light, dark = hex_rgb(0x5CB85C), hex_rgb(0x2F7D3A)
    for y in range(grass_y, S):
        d.line([(0, y), (S, y)], fill=lerp(light, dark, (y - grass_y) / (S - grass_y)))
    return img


def foreground():
    img = Image.new("RGBA", (S, S))
    bat = Image.open(sprites / "bat.png").convert("RGBA")
    bat = bat.resize((int(bat.width * 1.45), int(bat.height * 1.45)), Image.LANCZOS)
    bat = bat.rotate(-38, expand=True, resample=Image.BICUBIC)
    img.alpha_composite(bat, (S // 2 - bat.width // 2 + 130, S // 2 - bat.height // 2 + 10))

    head = Image.open(sprites / "head.png").convert("RGBA")
    head = head.resize((400, 400), Image.LANCZOS)
    img.alpha_composite(head, (S // 2 - 250, S // 2 - 210))

    # Ball, inked like the in-game one.
    d = ImageDraw.Draw(img)
    cx, cy, r = S // 2 + 150, S // 2 + 160, 58
    d.ellipse([cx - r - 9, cy - r - 9, cx + r + 9, cy + r + 9], fill=hex_rgb(0x18213D))
    d.ellipse([cx - r, cy - r, cx + r, cy + r], fill=hex_rgb(0x8E1219))
    d.ellipse([cx - r * 0.95, cy - r * 0.95, cx + r * 0.7, cy + r * 0.7], fill=hex_rgb(0xC81D25))
    d.arc([cx - r * 0.55, cy - r * 1.3, cx + r * 1.5, cy + r * 0.75], 120, 235, fill=hex_rgb(0xF6E7C6), width=7)

    # Shrink into the circle that round-masking launchers keep.
    k = 0.86
    small = img.resize((int(S * k), int(S * k)), Image.LANCZOS)
    img = Image.new("RGBA", (S, S))
    img.alpha_composite(small, ((S - small.width) // 2, (S - small.height) // 2))
    return img


def save(img, path, size):
    path.parent.mkdir(parents=True, exist_ok=True)
    img.resize((size, size), Image.LANCZOS).save(path)


bg, fg = background(), foreground()

# Legacy icon: the 72dp visible window of the adaptive layers, rounded.
inset = S // 6
legacy = Image.alpha_composite(bg, fg).crop((inset, inset, S - inset, S - inset))
mask = Image.new("L", legacy.size)
ImageDraw.Draw(mask).rounded_rectangle([0, 0, *legacy.size], radius=legacy.width // 6, fill=255)
legacy.putalpha(mask)

for density, scale in [("mdpi", 1), ("hdpi", 1.5), ("xhdpi", 2), ("xxhdpi", 3), ("xxxhdpi", 4)]:
    save(legacy, res / f"mipmap-{density}" / "ic_launcher.png", int(48 * scale))
    save(bg, res / f"mipmap-{density}" / "ic_launcher_background.png", int(108 * scale))
    save(fg, res / f"mipmap-{density}" / "ic_launcher_foreground.png", int(108 * scale))

(res / "mipmap-anydpi-v26").mkdir(parents=True, exist_ok=True)
(res / "mipmap-anydpi-v26" / "ic_launcher.xml").write_text(
    """<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@mipmap/ic_launcher_background" />
    <foreground android:drawable="@mipmap/ic_launcher_foreground" />
</adaptive-icon>
"""
)
legacy.save(root / "assets" / "android" / "icon-preview.png")

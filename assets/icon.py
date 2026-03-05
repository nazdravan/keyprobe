"""KeyProbe's mark: four keycaps in the app's key states (two tested, one held, one untested),
drawn from assets/theme.toml. Run from anywhere: python3 assets/icon.py

Outputs (all generated, never edited by hand):
  assets/icon.png            256x256, window icon, Linux .desktop, favicon
  assets/keyprobe.ico        16-256, Windows .exe resource
  assets/keyprobe.icns       16-1024, mark at ~80 % of the square (Apple grid)
  docs/assets/keyprobe-mark.png  transparent, README header
  docs/assets/icon-preview.png   the icon at 16, 32 and 256 px, for checking it still reads
"""
from PIL import Image, ImageDraw, ImageFont

import theme

N = 1024  # drawn large, then scaled down


def keycaps(size, t):
    """The 2x2 keycap grid on a transparent square, in the on-screen keyboard's flat style."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    gap = size * 0.08
    cap = (size - gap) / 2
    radius = cap * 0.18
    font = ImageFont.truetype(theme.ASSETS / "fonts" / "Ubuntu-Light.ttf", round(cap * 0.62))
    fills = [
        (t["accent"], None, "K"),
        (t["accent"], None, "P"),
        (t["held"], None, None),
        (t["key_dark"], t["accent"], None),
    ]
    for i, (fill, outline, legend) in enumerate(fills):
        x = (i % 2) * (cap + gap)
        y = (i // 2) * (cap + gap)
        box = [x, y, x + cap - 1, y + cap - 1]
        if outline:
            w = max(1, round(cap * 0.07))
            d.rounded_rectangle(box, radius, fill=theme.rgba(fill), outline=theme.rgba(outline), width=w)
        else:
            d.rounded_rectangle(box, radius, fill=theme.rgba(fill))
        if legend:
            d.text((x + cap / 2, y + cap / 2), legend, font=font, fill=theme.rgba(t["ink"]), anchor="mm")
    return img


def badge(t):
    """The keycaps on a dark rounded tile: the app icon."""
    img = Image.new("RGBA", (N, N), (0, 0, 0, 0))
    ImageDraw.Draw(img).rounded_rectangle([0, 0, N - 1, N - 1], round(N * 0.22), fill=theme.rgba(t["bg_dark"]))
    inner = round(N * 0.64)
    off = (N - inner) // 2
    img.alpha_composite(keycaps(inner, t), (off, off))
    return img


def main():
    t = theme.load()
    out = theme.ASSETS
    docs = theme.ROOT / "docs" / "assets"
    docs.mkdir(parents=True, exist_ok=True)

    tile = badge(t)
    icon = tile.resize((256, 256), Image.LANCZOS)
    icon.save(out / "icon.png", optimize=True)
    icon.save(out / "keyprobe.ico", sizes=[(s, s) for s in (16, 24, 32, 48, 64, 128, 256)])

    mac = Image.new("RGBA", (N, N), (0, 0, 0, 0))
    inner = round(N * 824 / 1024)
    mac.paste(tile.resize((inner, inner), Image.LANCZOS), ((N - inner) // 2, (N - inner) // 2))
    mac.save(out / "keyprobe.icns", sizes=[(s, s) for s in (16, 32, 64, 128, 256, 512, 1024)])

    keycaps(N, t).resize((260, 260), Image.LANCZOS).save(docs / "keyprobe-mark.png", optimize=True)

    sizes = (16, 32, 256)
    preview = Image.new("RGBA", (sum(sizes) + 40, 256 + 20), theme.rgba(t["bg_light"]))
    x = 10
    for s in sizes:
        preview.alpha_composite(tile.resize((s, s), Image.LANCZOS), (x, 10))
        x += s + 10
    preview.save(docs / "icon-preview.png", optimize=True)


if __name__ == "__main__":
    main()

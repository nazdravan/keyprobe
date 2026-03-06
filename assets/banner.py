"""README banner: KEYPROBE spelled on keycaps drawn like the app's on-screen keyboard, mid-test
(some keys tested, one held, the rest untested), from assets/theme.toml.

    python3 assets/banner.py

Outputs (generated, never edited by hand):
  docs/assets/banner.png   2430x420, transparent, reads on GitHub light and dark
"""
from PIL import Image, ImageDraw, ImageFont

import theme

W, H = 2430, 420
WORD = "KEYPROBE"
# Per letter: tested, held or untested, as the keyboard would show a test in progress.
STATES = ["tested", "tested", "tested", "held", "tested", "untested", "untested", "untested"]


def draw(t, width=W, height=H):
    img = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    gap = width * 0.012
    margin = width * 0.03
    cap = min((width - 2 * margin - gap * (len(WORD) - 1)) / len(WORD), height * 0.9)
    total = cap * len(WORD) + gap * (len(WORD) - 1)
    x0 = (width - total) / 2
    y0 = (height - cap) / 2
    radius = cap * 0.12
    border = max(2, round(cap * 0.012))
    font = ImageFont.truetype(theme.ASSETS / "fonts" / "Ubuntu-Light.ttf", round(cap * 0.56))
    for i, (letter, state) in enumerate(zip(WORD, STATES)):
        x = x0 + i * (cap + gap)
        box = [x, y0, x + cap, y0 + cap]
        if state == "tested":
            fill, text, outline = t["accent"], t["ink"], None
        elif state == "held":
            # The held key also carries the "last pressed" outline, as in the app.
            fill, text, outline = t["held"], t["ink"], t["accent"]
        else:
            fill, text, outline = t["key_dark"], t["fg_dark"], None
        if outline:
            d.rounded_rectangle(box, radius, fill=theme.rgba(fill), outline=theme.rgba(outline), width=border * 3)
        else:
            d.rounded_rectangle(box, radius, fill=theme.rgba(fill))
        d.text((x + cap / 2, y0 + cap / 2), letter, font=font, fill=theme.rgba(text), anchor="mm")
    return img


def main():
    out = theme.ROOT / "docs" / "assets"
    out.mkdir(parents=True, exist_ok=True)
    draw(theme.load()).save(out / "banner.png", optimize=True)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Marian PDFCraft's app icon: a solid pink rounded square (235, 6, 210) with its top-left corner
turned down in light pink (252, 206, 247). Emma's design, drawn in code. Writes
assets/app-icon/marian-pdfcraft-*.png and .icns. Needs Pillow. Run from anywhere:
python3 packaging/marian/icon.py"""
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets/app-icon"
PINK = (235, 6, 210)
LIGHT = (252, 206, 247)
S = 4096  # drawn large, then scaled down smoothly


def tile(size, inset):
    """The icon at `size` px, its square `inset` (of 1024) in from each edge (macOS leaves a margin)."""
    m = inset * S // 1024
    side = S - 2 * m
    fold = int(side * 0.36)
    body = Image.new("L", (S, S), 0)
    d = ImageDraw.Draw(body)
    d.rounded_rectangle([m, m, S - m - 1, S - m - 1], radius=int(side * 0.21875), fill=255)
    d.polygon([(m - 1, m - 1), (m + fold, m - 1), (m - 1, m + fold)], fill=0)
    im = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    im.paste(Image.new("RGBA", (S, S), PINK + (255,)), (0, 0), body)
    ImageDraw.Draw(im).polygon([(m + fold, m), (m + fold, m + fold), (m, m + fold)], fill=LIGHT + (255,))
    return im.resize((size, size), Image.LANCZOS)


tile(256, 0).save(OUT / "marian-pdfcraft-256.png")
mac = tile(1024, 100)
mac.save(OUT / "marian-pdfcraft-1024.png")
mac.save(OUT / "marian-pdfcraft.icns", sizes=[(16, 16), (32, 32), (64, 64), (128, 128), (256, 256), (512, 512), (1024, 1024)])
print("wrote", *sorted(p.name for p in OUT.glob("marian-pdfcraft*")))

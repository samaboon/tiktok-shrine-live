#!/usr/bin/env python3
"""Side-by-side 'ฉากเต็ม' vs 'โปร่งใส' live composites for each mode -> screenshots/live_bg_compare_*.png"""
import os, sys
sys.path.insert(0, os.path.dirname(__file__))
from make_composites import render, composite, F  # noqa: E402  (chdirs to repo root)
from PIL import Image, ImageDraw, ImageFont

MODES = {
    'shrine':   ('--demo', 'pray:3:มะลิ', '--demo', 'siamsi:2:สมชาย', '--demo', 'pray:1:Nok', '--seed', '5', '--shot-at', '9.5'),
    'krathong': ('--theme', 'krathong', '--demo', 'siamsi:4:เจ้าสัวเฮง', '--demo', 'pray:2:สมชาย', '--demo', 'pray:1:Nok', '--seed', '95', '--shot-at', '10.0'),
    'cat':      ('--theme', 'cat', '--demo', 'pray:3:มะลิ', '--demo', 'pray:2:สมชาย', '--demo', 'pray:1:Nok', '--seed', '42', '--shot-at', '7.4'),
    'idle':     ('--shot-at', '3'),
}
extra = sys.argv[1:]  # e.g. --props off
f = ImageFont.truetype(F, 30)
for mode, args in MODES.items():
    panels = []
    for bg, label in (('scene', 'ฉากเต็ม'), ('transparent', 'โปร่งใส')):
        raw = render(f'{mode}_{bg}', *args, '--bg', bg, *extra, opaque=(bg == 'scene'))
        im = composite(raw, None)
        d = ImageDraw.Draw(im, 'RGBA')
        tw = d.textlength(label, font=f)
        d.rounded_rectangle([12, 920 - 46, 12 + tw + 32, 920], radius=20, fill=(0, 0, 0, 170))
        d.text((28, 920 - 42), label, font=f, fill=(255, 230, 140))
        panels.append(im)
    out = Image.new('RGB', (1096, 960), (20, 20, 20))
    out.paste(panels[0], (0, 0))
    out.paste(panels[1], (556, 0))
    p = f'screenshots/live_bg_compare_{mode}.png'
    out.save(p)
    print('saved', p)

#!/usr/bin/env python3
"""Composite raw overlay screenshots onto a fake 9:16 'streamer' frame -> screenshots/live_*.png"""
from PIL import Image, ImageDraw, ImageFilter, ImageFont
import os
os.chdir(os.path.join(os.path.dirname(__file__), '..'))
W, H = 1080, 1920
F = 'assets/fonts/NotoSans-Bold.ttf'

def streamer_bg():
    """Mock streamer frame: room + a person whose face sits in the LOWER half (y > 960)."""
    bg = Image.new('RGB', (W, H))
    d = ImageDraw.Draw(bg)
    for y in range(H):  # cozy room gradient
        t = y / H
        d.line([(0, y), (W, y)], fill=(int(60 + 80 * t), int(50 + 40 * t), int(90 - 30 * t)))
    d.rectangle([80, 300, 420, 760], fill=(110, 130, 170))   # window
    d.rectangle([95, 315, 405, 745], fill=(170, 200, 235))
    d.rectangle([640, 620, 1000, 640], fill=(90, 60, 40))    # shelf
    for i, c in enumerate([(220, 90, 90), (90, 160, 220), (240, 200, 90), (120, 200, 120)]):
        d.rectangle([660 + i * 80, 540, 720 + i * 80, 620], fill=c)
    p = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    pd = ImageDraw.Draw(p)
    Y = 140  # whole person shifted down so the face is fully below the overlay band
    pd.rounded_rectangle([180, 1260 + Y, 900, 1920 + Y], radius=200, fill=(230, 80, 110, 255))  # body
    pd.rectangle([490, 1230 + Y, 590, 1300 + Y], fill=(225, 185, 155, 255))                    # neck
    pd.ellipse([350, 850 + Y, 730, 1100 + Y], fill=(40, 30, 30, 255))                           # hair
    pd.ellipse([385, 930 + Y, 695, 1250 + Y], fill=(240, 200, 170, 255))                        # face
    pd.pieslice([370, 860 + Y, 710, 1060 + Y], 180, 360, fill=(40, 30, 30, 255))               # fringe
    for ex in (455, 585):
        pd.ellipse([ex, 1070 + Y, ex + 40, 1110 + Y], fill=(255, 255, 255, 255))
        pd.ellipse([ex + 9, 1076 + Y, ex + 33, 1104 + Y], fill=(40, 30, 30, 255))
        pd.line([(ex - 4, 1052 + Y), (ex + 44, 1046 + Y)], fill=(40, 30, 30, 255), width=7)
    pd.ellipse([420, 1140 + Y, 470, 1170 + Y], fill=(255, 150, 150, 120))
    pd.ellipse([610, 1140 + Y, 660, 1170 + Y], fill=(255, 150, 150, 120))
    pd.arc([490, 1120 + Y, 590, 1190 + Y], 20, 160, fill=(170, 60, 70, 255), width=8)
    bg.paste(p, (0, 0), p)
    return bg.filter(ImageFilter.GaussianBlur(1.0))

def tiktok_ui(img):
    d = ImageDraw.Draw(img, 'RGBA')
    f = ImageFont.truetype(F, 34)
    fs = ImageFont.truetype(F, 30)
    # bottom chat
    msgs = [('somchai99', 'sent Rose x1'), ('mali.th', 'joined'), ('ton_rich', 'sent Lion!'), ('nana', '555+')]
    y = 1500
    for u, m in msgs:
        d.rounded_rectangle([24, y, 620, y + 64], radius=32, fill=(0, 0, 0, 90))
        d.text((44, y + 12), u, font=fs, fill=(255, 220, 120))
        d.text((44 + d.textlength(u, font=fs) + 16, y + 12), m, font=fs, fill=(255, 255, 255))
        y += 80
    # right icons
    for i in range(4):
        d.ellipse([970, 1300 + i * 120, 1050, 1380 + i * 120], fill=(255, 255, 255, 60))
    # LIVE pill
    d.rounded_rectangle([W - 190, H - 120, W - 30, H - 50], radius=30, fill=(254, 44, 85, 230))
    d.text((W - 150, H - 108), 'LIVE', font=f, fill=(255, 255, 255))
    return img

def composite(raw, out, green=False, full=False):
    ov = Image.open(raw).convert('RGBA')
    if green:  # simple chroma key
        px = ov.load()
        for y in range(ov.height):
            for x in range(ov.width):
                r, g, b, a = px[x, y]
                if g > 150 and g > r + 60 and g > b + 60:
                    px[x, y] = (0, 0, 0, 0)
                elif g > r and g > b:  # despill
                    px[x, y] = (r, max(r, b), b, a)
    tw = W
    th = int(ov.height * W / ov.width)
    ov = ov.resize((tw, th), Image.LANCZOS)
    img = streamer_bg().convert('RGBA')
    img.alpha_composite(ov, (0, 0))
    img = tiktok_ui(img)
    # guide line for top half
    if not full:
        d = ImageDraw.Draw(img, 'RGBA')
        for x in range(0, W, 40):
            d.line([(x, 960), (x + 20, 960)], fill=(255, 255, 255, 110), width=3)
    img.convert('RGB').resize((540, 960), Image.LANCZOS).save(out)
    print('saved', out)

import subprocess, tempfile
BIN = os.environ.get('BIN', './target/release/tiktok-shrine-live')
R = tempfile.mkdtemp(prefix='live_raw_') + '/'

def render(name, *args, opaque=False):
    """Render a TRUE-alpha RGBA frame (no checkerboard) with the app's headless shot mode."""
    out = R + name + '.png'
    subprocess.run([BIN, '--no-status', '--out', out, *args], check=True, timeout=120,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    im = Image.open(out)
    assert opaque or (im.mode == 'RGBA' and im.getextrema()[3][0] == 0), f'{name}: no real transparency'
    return out

D5 = 'pray:5:จักรพรรดิเพชร'
jobs = [
    ('live_pray_t5', ('--demo', D5, '--seed', '55', '--shot-at', '10.2'), {}),
    ('live_pray_t1', ('--demo', 'pray:1:มะลิ', '--seed', '11', '--shot-at', '9.2'), {}),
    ('live_siamsi_t3', ('--demo', 'siamsi:3:เศรษฐีต้น', '--seed', '39', '--shot-at', '11.4'), {}),
    ('live_krathong_t4', ('--theme', 'krathong', '--demo', 'pray:4:เจ้าสัวเฮง', '--seed', '68', '--shot-at', '7.3'), {}),
    ('live_krathong_t5_bless', ('--theme', 'krathong', '--demo', 'siamsi:5:จักรพรรดิเพชร', '--seed', '95', '--shot-at', '10.0'), {}),
    ('live_idle_still', ('--shot-at', '3'), {}),
    ('live_fullscreen_t5', ('--demo', D5, '--seed', '55', '--shot-at', '10.2', '--layout', 'full'), {'full': True}),
    ('live_multi_3senders', ('--demo', 'pray:3:มะลิ', '--demo', 'siamsi:2:สมชาย', '--demo', 'pray:1:Nok', '--seed', '5', '--shot-at', '9.5'), {}),
    ('live_cat_3senders', ('--theme', 'cat', '--demo', 'pray:3:มะลิ', '--demo', 'pray:2:สมชาย', '--demo', 'pray:1:Nok', '--seed', '42', '--shot-at', '7.4'), {}),
    ('live_green_keyed_t5', ('--demo', D5, '--seed', '55', '--shot-at', '10.2', '--bg', 'green'), {'green': True}),
]
for name, args, kw in jobs:
    composite(render(name, *args, opaque=kw.get('green', False)), f'screenshots/{name}.png', **kw)

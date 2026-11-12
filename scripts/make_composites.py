#!/usr/bin/env python3
"""Composite raw overlay screenshots onto a fake 9:16 'streamer' frame -> screenshots/live_*.png"""
from PIL import Image, ImageDraw, ImageFilter, ImageFont
import os
os.chdir(os.path.join(os.path.dirname(__file__), '..'))
W, H = 1080, 1920
F = 'assets/fonts/NotoSans-Bold.ttf'

def streamer_bg():
    bg = Image.new('RGB', (W, H))
    d = ImageDraw.Draw(bg)
    for y in range(H):  # cozy room gradient
        t = y / H
        d.line([(0, y), (W, y)], fill=(int(60 + 80 * t), int(50 + 40 * t), int(90 - 30 * t)))
    # shelves / window light
    d.rectangle([80, 300, 420, 760], fill=(110, 130, 170))
    d.rectangle([95, 315, 405, 745], fill=(170, 200, 235))
    d.rectangle([640, 520, 1000, 540], fill=(90, 60, 40))
    for i, c in enumerate([(220, 90, 90), (90, 160, 220), (240, 200, 90), (120, 200, 120)]):
        d.rectangle([660 + i * 80, 440, 720 + i * 80, 520], fill=c)
    # streamer silhouette
    p = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    pd = ImageDraw.Draw(p)
    pd.ellipse([370, 880, 710, 1260], fill=(240, 200, 170, 255))
    pd.ellipse([350, 830, 730, 1080], fill=(40, 30, 30, 255))
    pd.ellipse([385, 930, 695, 1250], fill=(240, 200, 170, 255))
    pd.rounded_rectangle([180, 1240, 900, 1920], radius=200, fill=(230, 80, 110, 255))
    pd.ellipse([460, 1060, 490, 1090], fill=(40, 30, 30, 255))
    pd.ellipse([590, 1060, 620, 1090], fill=(40, 30, 30, 255))
    pd.arc([480, 1110, 600, 1180], 20, 160, fill=(150, 60, 60, 255), width=8)
    bg.paste(p, (0, 0), p)
    bg = bg.filter(ImageFilter.GaussianBlur(1.2))
    return bg

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

R = 'screenshots/raw/'
composite(R + 'pray_t5_4_fireworks.png', 'screenshots/live_pray_t5.png')
composite(R + 'pray_t1_3_number.png', 'screenshots/live_pray_t1.png')
composite(R + 'siamsi_t3_3_fortune.png', 'screenshots/live_siamsi_t3.png')
composite(R + 'mode_green_t5.png', 'screenshots/live_green_keyed_t5.png', green=True)
composite(R + 'layout_full_t5.png', 'screenshots/live_fullscreen_t5.png', full=True)

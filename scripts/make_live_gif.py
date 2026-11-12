#!/usr/bin/env python3
"""Live-composite GIF/MP4: true-alpha recorded frames over the mock streamer frame (face visible below).
usage: make_live_gif.py <frames_dir> <out_basename>   e.g. previews/live_multi_3senders"""
import os, sys, subprocess, tempfile
here = os.path.dirname(os.path.abspath(__file__))
src = open(os.path.join(here, 'make_composites.py'), encoding='utf-8').read()
exec(src.split('import subprocess, tempfile')[0])   # streamer_bg / tiktok_ui / composite only
frames, out = sys.argv[1], sys.argv[2]
tmp = tempfile.mkdtemp(prefix='livegif_')
bg = tiktok_ui(streamer_bg().convert('RGBA'))
for i, f in enumerate(sorted(x for x in os.listdir(frames) if x.endswith('.png'))):
    ov = Image.open(os.path.join(frames, f)).convert('RGBA')
    ov = ov.resize((W, int(ov.height * W / ov.width)), Image.LANCZOS)
    img = bg.copy(); img.alpha_composite(ov, (0, 0))
    d = ImageDraw.Draw(img, 'RGBA')
    for x in range(0, W, 40):
        d.line([(x, 960), (x + 20, 960)], fill=(255, 255, 255, 110), width=3)
    img.convert('RGB').resize((540, 960), Image.LANCZOS).save(f'{tmp}/f{i:05d}.png')
pal = 'scale=300:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=128[p];[b][p]paletteuse=dither=bayer:bayer_scale=4'
subprocess.run(['ffmpeg', '-loglevel', 'error', '-y', '-framerate', '10', '-i', f'{tmp}/f%05d.png', '-vf', pal, out + '.gif'], check=True)
subprocess.run(['ffmpeg', '-loglevel', 'error', '-y', '-framerate', '10', '-i', f'{tmp}/f%05d.png', '-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-crf', '26', out + '.mp4'], check=True)
print('saved', out + '.gif', out + '.mp4')

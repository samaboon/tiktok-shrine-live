#!/usr/bin/env python3
"""End-to-end test of the overlay: keyboard keys (xdotool), HTTP routes, gift->action mapping,
tier thresholds (diamonds x repeat), queue, modes, idle, save/reload, start/stop, click-through.
Runs the real binary under Xvfb and reads its /state JSON. Writes tests/e2e_results.md."""
import json, os, re, shutil, subprocess, sys, time, urllib.request, urllib.parse
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
BIN = os.environ.get('BIN', os.path.join(ROOT, 'target/release/tiktok-shrine-live'))
PORT = 18795
BASE = f'http://127.0.0.1:{PORT}'
WORK = '/tmp/e2e_shrine'
results = []

def check(name, ok, detail=''):
    results.append((name, bool(ok), detail))
    print(('PASS ' if ok else 'FAIL ') + name + ('  -- ' + detail if detail else ''))

def get(path, data=None, ctype=None):
    req = urllib.request.Request(BASE + path, data=data)
    if ctype: req.add_header('Content-Type', ctype)
    with urllib.request.urlopen(req, timeout=5) as r:
        return r.status, r.read().decode()

def state():
    return json.loads(get('/state')[1])

def wait_for(pred, timeout=6.0):
    t0 = time.time()
    while time.time() - t0 < timeout:
        try:
            s = state()
            if pred(s): return s
        except Exception: pass
        time.sleep(0.15)
    return state()

def tiers_seen(s):
    out = []
    if s.get('current'): out.append(s['current']['tier'])
    out += [q['tier'] for q in s['queue']]
    return out

def clear_all():
    get('/control?clear=1')
    # wait until the current show ends is too slow -> restart-free: speed via queue clear only
    time.sleep(0.3)

def xkey(win, key):
    subprocess.run(['xdotool', 'windowfocus', '--sync', win], capture_output=True)
    subprocess.run(['xdotool', 'key', '--window', win, key], capture_output=True)
    time.sleep(0.35)

def main():
    shutil.rmtree(WORK, ignore_errors=True); os.makedirs(WORK)
    cfg = open(os.path.join(ROOT, 'config.toml'), encoding='utf-8').read()
    cfg = re.sub(r'(?m)^bind = .*', f'bind = "127.0.0.1:{PORT}"', cfg)
    cfg = re.sub(r'(?m)^check_on_start = .*', 'check_on_start = false', cfg)
    cfg = re.sub(r'(?m)^speed = .*', 'speed = 4.0', cfg)   # fast shows
    open(os.path.join(WORK, 'config.toml'), 'w', encoding='utf-8').write(cfg)
    env = dict(os.environ)
    log = open(os.path.join(WORK, 'app.log'), 'w')
    p = subprocess.Popen([BIN, '--no-status'], cwd=WORK, stdout=log, stderr=subprocess.STDOUT, env=env)
    try:
        s = wait_for(lambda s: 'frame' in s, 15)
        check('app starts, /state responds', 'frame' in s, f"version {s.get('version')}")
        check('GET /health', get('/health') == (200, 'ok'))
        check('idle at start (still picture, no character)', s['idle'] and s['current'] is None and s['idle_mode'] == 'still')
        t1 = state()['scene_time']; time.sleep(1.0); t2 = state()['scene_time']
        check('still idle: scene frozen (no moving effects)', abs(t2 - t1) < 1e-4, f'{t1:.3f} -> {t2:.3f}')

        # ---------------- keyboard
        win = subprocess.run(['xdotool', 'search', '--name', '^TikTok Shrine Live$'], capture_output=True, text=True).stdout.split()
        if not win:
            check('find overlay window (xdotool)', False)
        else:
            w = win[0]
            before = state()['shows_started'] + state()['queue_len']
            keys = [('1', 'pray test tier'), ('2', 'siamsi test tier'), ('q', 'pray t1'), ('w', 'pray t2'), ('e', 'pray t3'), ('r', 'pray t4'), ('t', 'pray t5'),
                    ('a', 'siamsi t1'), ('s', 'siamsi t2'), ('d', 'siamsi t3'), ('f', 'siamsi t4'), ('g', 'siamsi t5')]
            for k, label in keys:
                n0 = state(); c0 = n0['shows_started'] + n0['queue_len']
                xkey(w, k)
                n1 = wait_for(lambda s: s['shows_started'] + s['queue_len'] > c0, 3)
                check(f'key {k.upper()} -> {label}', n1['shows_started'] + n1['queue_len'] == c0 + 1)
            q = state()
            want = [1, 2, 3, 4, 5]
            got = [x['tier'] for x in q['queue'] if x['action'] == 'Pray'][-5:]
            check('Q W E R T map to tiers 1..5', got == want, str(got))
            got = [x['tier'] for x in q['queue'] if x['action'] == 'Siamsi'][-5:]
            check('A S D F G map to tiers 1..5', got == want, str(got))
            xkey(w, 'shift+3'); check('Shift+3 selects test tier 3', state()['test_tier'] == 3)
            xkey(w, 'Delete'); check('Delete clears the queue', state()['queue_len'] == 0)
            ct = state()['click_through']; xkey(w, 'F8'); check('F8 toggles click-through', state()['click_through'] != ct)
            xkey(w, 'F8')
            bg = state()['background']; xkey(w, 'F9'); bg2 = state()['background']; check('F9 cycles background', bg2 != bg, f'{bg} -> {bg2}')
            xkey(w, 'F9'); xkey(w, 'F9')
            th = state()['theme']; xkey(w, 'F2'); th2 = state()['theme']; check('F2 switches mode', th != th2, f'{th} -> {th2}')
            xkey(w, 'F2')
            so = state()['settings_open']; xkey(w, 'F1'); check('F1 opens/closes settings', state()['settings_open'] != so)
            xkey(w, 'F1')
        clear_all()

        # ---------------- HTTP event routes
        def count(): s = state(); return s['shows_started'] + s['queue_len']
        c = count(); get('/event?action=pray&user=%E0%B8%AA%E0%B8%A1%E0%B8%8A%E0%B8%B2%E0%B8%A2&diamonds=1')
        s = wait_for(lambda s: s['shows_started'] + s['queue_len'] > c)
        check('GET /event?action=pray', count() == c + 1)
        c = count(); get('/gift?gift_name=TikTok&user=nana')
        wait_for(lambda s: s['shows_started'] + s['queue_len'] > c)
        q = state(); last = (q['queue'] or [q['current']])[-1]
        check('GET /gift TikTok -> siamsi (gift mapping)', last['action'] == 'Siamsi', str(last))
        c = count(); get('/gift?gift_id=5655&user=idonly')
        wait_for(lambda s: s['shows_started'] + s['queue_len'] > c)
        q = state(); last = (q['queue'] or [q['current']])[-1]
        check('gift id 5655 -> pray (mapping by id)', last['action'] == 'Pray')
        c = count(); get('/event', json.dumps({'action': 'siamsi', 'user': 'json-user', 'diamonds': 5}).encode(), 'application/json')
        wait_for(lambda s: s['shows_started'] + s['queue_len'] > c)
        check('POST /event JSON', count() == c + 1)
        c = count(); get('/webhook', json.dumps({'event': 'gift', 'giftName': 'Rose', 'diamondCount': 1, 'repeatCount': 1, 'nickname': 'tikfinity'}).encode(), 'application/json')
        wait_for(lambda s: s['shows_started'] + s['queue_len'] > c)
        check('POST /webhook (TikFinity style JSON)', count() == c + 1)
        c = count(); get('/gift?gift_name=UnknownGiftXYZ&user=x')
        wait_for(lambda s: s['shows_started'] + s['queue_len'] > c)
        q = state(); last = (q['queue'] or [q['current']])[-1]
        check('unknown gift -> default action (pray)', count() == c + 1 and last['action'] == 'Pray')

        # ---------------- tiers (diamonds x repeat)
        clear_all()
        cases = [(1, 1, 1), (9, 1, 1), (10, 1, 2), (99, 1, 2), (100, 1, 3), (999, 1, 3), (1000, 1, 4), (9999, 1, 4), (10000, 1, 5), (1, 10, 2), (5, 20, 3), (1, 1000, 4), (500, 20, 5)]
        ok_all = True; det = []
        for d, rep, want in cases:
            get('/control?clear=1'); time.sleep(0.1)
            g0 = state()['gifts_received']
            get(f'/gift?gift_name=Rose&user=tier{d}x{rep}&diamonds={d}&repeat={rep}')
            s = wait_for(lambda s: s['gifts_received'] > g0 and any(e and e['user'] == f'tier{d}x{rep}' for e in [s['current']] + s['queue']), 3)
            ent = [e for e in [s['current']] + s['queue'] if e and e['user'] == f'tier{d}x{rep}']
            got = ent[-1]['tier'] if ent else None
            det.append(f'{d}x{rep}->{got}')
            if got != want: ok_all = False
        check('5 tier thresholds incl. diamonds x repeat (0/10/100/1000/10000)', ok_all, ', '.join(det))
        get('/control?clear=1')
        get('/gift?gift_name=Rose&user=combo&diamonds=1&repeat=5')
        s = wait_for(lambda s: any(e and e['user'] == 'combo' for e in [s['current']] + s['queue']), 3)
        ent = [e for e in [s['current']] + s['queue'] if e and e['user'] == 'combo']
        check('combo streak (repeat=5) plays once with summed value (max_plays_per_gift=1)', len(ent) == 1 and ent[0]['tier'] == 1, str(ent))

        # ---------------- 10 rapid gifts
        get('/control?clear=1')
        for i in range(10):
            get(f'/gift?gift_name=Rose&user=fan{i}&diamonds=1')
        def fans(s): return [e['user'] for e in [s['current']] + s['queue'] if e and e['user'].startswith('fan')]
        s = wait_for(lambda s: len(fans(s)) >= 10, 4)
        check('10 rapid gifts all queued in order', fans(s) == [f'fan{i}' for i in range(10)], f"queue_len={s['queue_len']} {fans(s)}")
        get('/control?clear=1')

        # ---------------- 3 senders at once -> visible single-file line, front one plays, line advances
        s = wait_for(lambda s: s['current'] is None, 15)
        q = urllib.parse.quote
        for u in ['ผู้ส่งA', 'ผู้ส่งB', 'ผู้ส่งC🌸']:
            get(f'/gift?gift_name=Rose&user={q(u)}&diamonds=1')
        s = wait_for(lambda s: s['current'] is not None and len(s['line']) == 2, 4)
        time.sleep(1.2); s = state()
        line = s['line']; xs = [e['x'] for e in line]
        check('3 senders: front one plays, other two visible in line', s['current'] and s['current']['user'] == 'ผู้ส่งA' and [e['user'] for e in line] == ['ผู้ส่งB', 'ผู้ส่งC'], f"current={s['current'] and s['current']['user']} line={line}")
        check('3 senders: line is single-file, no overlap (>=100 px apart, on screen)', len(xs) == 2 and xs[0] - xs[1] >= 100 and min(xs) >= 0, f'x={xs}')
        check('emoji stripped from name label (no tofu)', any(e['user'] == 'ผู้ส่งC' for e in line))
        s = wait_for(lambda s: s['current'] is not None and s['current']['user'] == 'ผู้ส่งB', 15)
        check('3 senders: line advances (2nd sender steps up, 3rd moves forward)', s['current']['user'] == 'ผู้ส่งB' and [e['user'] for e in s['line']] == ['ผู้ส่งC'], f"line={s['line']}")
        s = wait_for(lambda s: s['current'] is not None and s['current']['user'] == 'ผู้ส่งC', 15)
        check('3 senders: each gets their own turn in order', s['current']['user'] == 'ผู้ส่งC' and s['line'] == [])
        get('/control?clear=1')

        # ---------------- controls
        get('/control?layout=full'); s = wait_for(lambda s: s['layout'] == 'full')
        wfull = s['window_size']
        get('/control?layout=top'); s = wait_for(lambda s: s['layout'] == 'top')
        check('layout toggle full/top (window resizes)', wfull != s['window_size'], f'{wfull} vs {s["window_size"]}')
        for b in ['green', 'scene', 'transparent']:
            get(f'/control?background={b}'); s = wait_for(lambda s: s['background'] == b)
            check(f'background mode {b}', s['background'] == b)
        get('/control?mode=krathong'); s = wait_for(lambda s: s['theme'] == 'krathong')
        check('runtime mode switch -> krathong', s['theme'] == 'krathong')
        c = count(); get('/gift?gift_name=Rose&user=loy&diamonds=1000')
        s = wait_for(lambda s: s['floating'] >= 1, 12)
        check('krathong: krathong is released and keeps floating', s['floating'] >= 1, f"floating={s['floating']}")
        get('/control?mode=shrine'); s = wait_for(lambda s: s['theme'] == 'shrine')
        check('runtime mode switch -> shrine', s['theme'] == 'shrine')
        get('/control?click_through=1'); s = wait_for(lambda s: s['click_through'])
        get('/control?click_through=0'); s2 = wait_for(lambda s: not s['click_through'])
        check('click-through on/off via /control', s['click_through'] and not s2['click_through'])
        get('/control?running=0'); s = wait_for(lambda s: not s['running'])
        get('/control?clear=1'); time.sleep(0.2)
        g0 = state(); get('/gift?gift_name=Rose&user=ignored'); time.sleep(0.5); g1 = state()
        check('stop: gifts ignored while stopped', not g1['running'] and g1['queue_len'] == g0['queue_len'] and g1['gifts_received'] == g0['gifts_received'] + 1)
        get('/control?running=1'); s = wait_for(lambda s: s['running'])
        c = count(); get('/gift?gift_name=Rose&user=back'); wait_for(lambda s: s['shows_started'] + s['queue_len'] > c)
        check('start: gifts accepted again', count() == c + 1)
        get('/control?clear=1')

        # ---------------- idle modes
        get('/control?idle=hidden')
        s = wait_for(lambda s: s['idle'] and s['scene_alpha'] < 0.01, 40)
        check('idle=hidden: scene fades fully transparent when idle', s['idle'] and s['scene_alpha'] < 0.01, f"alpha={s['scene_alpha']:.2f}")
        c = count(); get('/gift?gift_name=Rose&user=wake')
        s = wait_for(lambda s: s['current'] is not None, 5)
        check('idle=hidden: gift fades the scene back in and plays', s['current'] is not None and s['scene_alpha'] > 0.9)
        get('/control?clear=1'); get('/control?idle=ambient')
        s = wait_for(lambda s: s['idle'], 40)
        t1 = state()['scene_time']; time.sleep(0.8); t2 = state()['scene_time']
        check('idle=ambient: subtle motion keeps running', t2 > t1)
        get('/control?idle=still'); time.sleep(1.6)
        t1 = state()['scene_time']; time.sleep(0.8); t2 = state()['scene_time']
        check('idle=still: back to frozen still picture', abs(t2 - t1) < 1e-4)

        # ---------------- save / reload round-trip
        cp = os.path.join(WORK, 'config.toml')
        get('/control?background=green'); get('/control?mode=krathong'); get('/control?idle=ambient'); time.sleep(0.4)
        get('/control?save=1'); time.sleep(0.6)
        txt = open(cp, encoding='utf-8').read()
        check('save writes config.toml', 'background = "green"' in txt and 'mode = "krathong"' in txt and 'idle = "ambient"' in txt)
        txt2 = txt.replace('background = "green"', 'background = "scene"').replace('mode = "krathong"', 'mode = "shrine"')
        open(cp, 'w', encoding='utf-8').write(txt2)
        get('/control?reload=1'); s = wait_for(lambda s: s['background'] == 'scene' and s['theme'] == 'shrine')
        check('reload picks up edited config.toml', s['background'] == 'scene' and s['theme'] == 'shrine' and s['idle_mode'] == 'ambient')
        thai = 'เลขเด็ด' in txt and 'คำอวยพร' in txt or 'ลอยกระทง' in txt
        check('saved config keeps Thai text (UTF-8)', thai)

        # ---------------- quit hotkey
        if win:
            subprocess.run(['xdotool', 'windowfocus', '--sync', win[0]], capture_output=True)
            subprocess.run(['xdotool', 'key', '--window', win[0], 'ctrl+q'], capture_output=True)
            try:
                p.wait(5); check('Ctrl+Q quits', True)
            except subprocess.TimeoutExpired:
                check('Ctrl+Q quits', False)
        log_txt = open(os.path.join(WORK, 'app.log'), encoding='utf-8', errors='replace').read()
        check('no panic in log', 'panicked' not in log_txt)
    finally:
        if p.poll() is None: p.kill()
    passed = sum(1 for r in results if r[1])
    os.makedirs(os.path.join(ROOT, 'tests'), exist_ok=True)
    with open(os.path.join(ROOT, 'tests/e2e_results.md'), 'w', encoding='utf-8') as f:
        f.write(f'# E2E results ({time.strftime("%Y-%m-%d %H:%M %Z")})\n\n{passed}/{len(results)} passed\n\n| # | test | result | detail |\n|---|---|---|---|\n')
        for i, (n, ok, d) in enumerate(results, 1):
            f.write(f'| {i} | {n} | {"PASS" if ok else "FAIL"} | {d} |\n')
    print(f'\n{passed}/{len(results)} passed')
    sys.exit(0 if passed == len(results) else 1)

main()

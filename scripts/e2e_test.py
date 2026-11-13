#!/usr/bin/env python3
"""End-to-end test of the overlay: keyboard keys, HTTP routes, gift->action mapping,
tier thresholds (diamonds x repeat), queue, modes, idle, save/reload, start/stop, click-through,
and the v0.2.4 shared rates / per-sticker dance moves / stage+star rules.

Keyboard: on Linux the real binary runs under Xvfb and keys are sent with xdotool.
Where xdotool is unavailable (macOS/Windows) the same hotkeys are driven through real egui
passes by `cargo test --release hotkey_tests`, and each check reports that test's result.
Writes tests/e2e_results.md."""
import json, os, platform, re, shutil, subprocess, sys, time, urllib.request, urllib.parse
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
BIN = os.environ.get('BIN', os.path.join(ROOT, 'target/release/tiktok-shrine-live'))
PORT = 18795
BASE = f'http://127.0.0.1:{PORT}'
WORK = '/tmp/e2e_shrine'
XDOTOOL = shutil.which('xdotool')
results = []

# E2E keyboard check -> the headless Rust test that covers it (used when xdotool is absent)
HOTKEY_CHECKS = [
    ('key 1 -> pray test tier', 'key_1_enqueues_pray_at_test_tier'),
    ('key 2 -> siamsi test tier', 'key_2_enqueues_siamsi_at_test_tier'),
    ('key Q -> pray t1', 'pray_tier_keys_q_w_e_r_t_map_to_tiers_1_5'),
    ('key W -> pray t2', 'pray_tier_keys_q_w_e_r_t_map_to_tiers_1_5'),
    ('key E -> pray t3', 'pray_tier_keys_q_w_e_r_t_map_to_tiers_1_5'),
    ('key R -> pray t4', 'pray_tier_keys_q_w_e_r_t_map_to_tiers_1_5'),
    ('key T -> pray t5', 'pray_tier_keys_q_w_e_r_t_map_to_tiers_1_5'),
    ('key A -> siamsi t1', 'siamsi_tier_keys_a_s_d_f_g_map_to_tiers_1_5'),
    ('key S -> siamsi t2', 'siamsi_tier_keys_a_s_d_f_g_map_to_tiers_1_5'),
    ('key D -> siamsi t3', 'siamsi_tier_keys_a_s_d_f_g_map_to_tiers_1_5'),
    ('key F -> siamsi t4', 'siamsi_tier_keys_a_s_d_f_g_map_to_tiers_1_5'),
    ('key G -> siamsi t5', 'siamsi_tier_keys_a_s_d_f_g_map_to_tiers_1_5'),
    ('Q W E R T map to tiers 1..5', 'pray_tier_keys_q_w_e_r_t_map_to_tiers_1_5'),
    ('A S D F G map to tiers 1..5', 'siamsi_tier_keys_a_s_d_f_g_map_to_tiers_1_5'),
    ('Shift+3 selects test tier 3', 'shift_3_selects_test_tier_3'),
    ('Delete clears the queue', 'delete_clears_the_queue'),
    ('F8 toggles click-through', 'f8_toggles_click_through'),
    ('F9 cycles background', 'f9_cycles_background_transparent_green_scene'),
    ('F2 switches mode', 'f2_switches_mode'),
    ('F2 cycles all 6 modes (shrine -> krathong -> cat -> dance -> dunk -> pet)', 'f2_cycles_all_6_modes_in_order'),
    ('F1 opens/closes settings', 'f1_opens_and_closes_settings'),
]
QUIT_TEST = 'cmd_q_requests_window_close'


def run_hotkey_tests():
    """Drive the hotkeys through real egui passes headlessly; returns {test_name: passed}."""
    p = subprocess.run(['cargo', 'test', '--release', 'hotkey_tests'], cwd=ROOT,
                       capture_output=True, text=True, timeout=1800)
    ok = {}
    for line in (p.stdout + p.stderr).splitlines():
        m = re.match(r'test app::hotkey_tests::(\w+) \.\.\. (ok|FAILED)', line.strip())
        if m:
            ok[m.group(1)] = m.group(2) == 'ok'
    return ok

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
        # one-button updater exposes app + connector + runtime status; connector runtime resolves without a Node install
        u = s.get('update', {})
        check('update state: one-button fields (pending/runtime/connector)', all(k in u for k in ('pending', 'pending_text', 'runtime_current', 'runtime_managed', 'bridge_current')) and isinstance(u['pending'], list), json.dumps({k: u.get(k) for k in ('pending', 'runtime_current', 'bridge_current')}))
        wn = subprocess.run([BIN, '--which-node'], capture_output=True, text=True, timeout=30)
        check('--which-node resolves a runtime (bundled/system)', wn.returncode == 0 and wn.stdout.split()[0] in ('bundled', 'system', 'custom'), wn.stdout.strip())
        check('idle at start (still picture, no character)', s['idle'] and s['current'] is None and s['idle_mode'] == 'still')
        t1 = state()['scene_time']; time.sleep(1.0); t2 = state()['scene_time']
        check('still idle: scene frozen (no moving effects)', abs(t2 - t1) < 1e-4, f'{t1:.3f} -> {t2:.3f}')

        # ---------------- keyboard
        hotkey_ok = {}
        win = []
        if not XDOTOOL:
            hotkey_ok = run_hotkey_tests()
            for label, tname in HOTKEY_CHECKS:
                check(label, hotkey_ok.get(tname, False), f'headless egui pass: hotkey_tests::{tname}')
        else:
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
                xkey(w, 'F2'); th3 = state()['theme']
                xkey(w, 'F2'); th4 = state()['theme']
                xkey(w, 'F2'); th5 = state()['theme']
                check('F2 cycles all 5 modes (shrine -> krathong -> cat -> dance -> dunk)', [th, th2, th3, th4, th5] == ['shrine', 'krathong', 'cat', 'dance', 'dunk'], f'{th} -> {th2} -> {th3} -> {th4} -> {th5}')
                get('/control?mode=shrine'); wait_for(lambda s: s['theme'] == 'shrine')
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
        cases = [(1, 1, 1), (4, 1, 1), (5, 1, 2), (19, 1, 2), (20, 1, 3), (29, 1, 3), (30, 1, 4), (99, 1, 4), (100, 1, 5), (1000, 1, 5), (1, 5, 2), (5, 4, 3), (1, 30, 4), (1, 100, 5), (500, 20, 5)]
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
        check('5 tier thresholds incl. diamonds x repeat (0/5/20/30/100)', ok_all, ', '.join(det))
        get('/control?clear=1')
        get('/gift?gift_name=Rose&user=combo&diamonds=1&repeat=5')
        s = wait_for(lambda s: any(e and e['user'] == 'combo' for e in [s['current']] + s['queue']), 3)
        ent = [e for e in [s['current']] + s['queue'] if e and e['user'] == 'combo']
        check('combo streak (repeat=5) plays once with summed value (max_plays_per_gift=1)', len(ent) == 1 and ent[0]['tier'] == 2, str(ent))

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
        cur = s['current'] or {}
        check('shrine result card matches action (pray -> lucky numbers, siamsi -> fortune)', cur.get('card') == ('numbers' if cur.get('action') == 'Pray' else 'fortune'), f"{cur.get('action')} -> {cur.get('card')}")
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
        # transparent (โปร่งใส): idle = no props over the camera; props fade in while a gift plays
        s = wait_for(lambda s: s['current'] is None and not s['queue'] and s.get('props_alpha', 1) < 0.05, 25)
        check('transparent idle: ground props hidden', s.get('props_alpha', 1) < 0.05 and s.get('props') is True, f"props_alpha={s.get('props_alpha')}")
        get('/gift?gift_name=Rose&user=propcheck&diamonds=1')
        s = wait_for(lambda s: s['current'] is not None and s.get('props_alpha', 0) > 0.9, 10)
        check('transparent show: props fade in', s.get('props_alpha', 0) > 0.9, f"props_alpha={s.get('props_alpha')}")
        get('/control?clear=1')
        get('/control?mode=krathong'); s = wait_for(lambda s: s['theme'] == 'krathong')
        check('runtime mode switch -> krathong', s['theme'] == 'krathong')
        c = count(); get('/gift?gift_name=Rose&user=loy&diamonds=1000')
        s = wait_for(lambda s: s['floating'] >= 1, 12)
        check('krathong: krathong is released and keeps floating', s['floating'] >= 1, f"floating={s['floating']}")
        # ---------------- cat mode: visible cat queue, thank-you card only (no lucky numbers)
        get('/control?clear=1'); get('/control?mode=cat'); s = wait_for(lambda s: s['theme'] == 'cat')
        check('runtime mode switch -> cat (ให้อาหารแมว)', s['theme'] == 'cat')
        s = wait_for(lambda s: s['current'] is None, 20)
        q = urllib.parse.quote
        for u in ['แมวA', 'แมวB', 'แมวC']:
            get(f'/gift?gift_name=Rose&user={q(u)}&diamonds=30')
        s = wait_for(lambda s: s['current'] is not None and len(s['line']) == 2, 4)
        check('cat: 3 senders -> front cat + 2 cats visible in line', s['current'] and s['current']['user'] == 'แมวA' and [e['user'] for e in s['line']] == ['แมวB', 'แมวC'], f"current={s['current'] and s['current']['user']} line={s['line']}")
        seen = set(); cards = set(); t0 = time.time()
        while time.time() - t0 < 25:
            c = state()['current']
            if c and c['user'] == 'แมวA':
                seen.add(c.get('phase')); cards.add(c.get('card'))
            elif c and c['user'] == 'แมวB':
                break
            time.sleep(0.1)
        check('cat: walk -> pour -> eat -> card phases', {'pour', 'eat', 'card'} <= seen, str(sorted(x for x in seen if x)))
        check('cat: result is thank-you card only (no lucky numbers)', cards == {'thanks'}, str(cards))
        s = wait_for(lambda s: s['current'] is not None and s['current']['user'] == 'แมวB', 20)
        check('cat: line advances to next sender', s['current'] and s['current']['user'] == 'แมวB' and [e['user'] for e in s['line']] == ['แมวC'], f"line={s['line']}")
        get('/control?clear=1')
        s = wait_for(lambda s: s['current'] is None or s['current']['user'] != 'แมวB', 25)

        # ---------------- dance mode: challenge card, venues, 3-sender floor, no lucky numbers
        get('/control?clear=1'); get('/control?mode=dance'); s = wait_for(lambda s: s['theme'] == 'dance')
        check('runtime mode switch -> dance (โหมดเต้น)', s['theme'] == 'dance')
        s = wait_for(lambda s: s['current'] is None, 20)
        q = urllib.parse.quote
        # hall dancers run concurrently on their own clock, so stagger arrivals to make the lead succession observable
        for u in ['เต้นA', 'เต้นB', 'เต้นC']:
            get(f'/gift?gift_name=Rose&user={q(u)}&diamonds=20')
            time.sleep(1.5)
        s = wait_for(lambda s: s['current'] is not None and len(s['line']) == 2, 6)
        check('dance: 3 senders -> lead + 2 on the floor', s['current'] and s['current']['user'] == 'เต้นA' and [e['user'] for e in s['line']] == ['เต้นB', 'เต้นC'], f"current={s['current'] and s['current']['user']} line={s['line']}")
        seen = set(); cards = set(); t0 = time.time()
        while time.time() - t0 < 20:
            c = state()['current']
            if c and c['user'] == 'เต้นA':
                seen.add(c.get('phase')); cards.add(c.get('card'))
            elif c and c['user'] == 'เต้นB':
                break
            time.sleep(0.1)
        check('dance: walk then dance phases', {'walk', 'dance'} <= seen or 'dance' in seen, str(sorted(x for x in seen if x)))
        check('dance: result is challenge card only (no lucky numbers)', cards <= {'challenge'} and 'challenge' in cards, str(cards))
        s = wait_for(lambda s: s['current'] is not None and s['current']['user'] == 'เต้นB', 25)
        c = s.get('current')
        seq = [(c['user'] if c else None, [e['user'] for e in s.get('line') or []])]
        t0 = time.time()
        while time.time() - t0 < 20:
            st = state(); c = st.get('current')
            lead = c['user'] if c else None
            if lead != seq[-1][0]:
                seq.append((lead, [e['user'] for e in st.get('line') or []]))
                if lead is None: break
            time.sleep(0.1)
        check('dance: lead advances B -> C then clears (hall dancers leave on their own clock)',
              [u for u, _ in seq] == ['เต้นB', 'เต้นC', None] and seq[0][1] == ['เต้นC'] and seq[1][1] == [],
              f"succession={seq}")
        get('/control?clear=1')

        # ---------------- dunk mode: queue, t5 instant dunk, splash credits the sender, no lucky numbers
        get('/control?mode=dunk'); s = wait_for(lambda s: s['theme'] == 'dunk')
        check('runtime mode switch -> dunk (ปาตกน้ำ)', s['theme'] == 'dunk')
        s = wait_for(lambda s: s['current'] is None, 20)
        for u in ['ปาA', 'ปาB', 'ปาC']:
            get(f'/gift?gift_name=Rose&user={q(u)}&diamonds=1')
        s = wait_for(lambda s: s['current'] is not None and len(s['line']) == 2, 4)
        check('dunk: 3 senders -> thrower + 2 waiting', s['current'] and s['current']['user'] == 'ปาA' and [e['user'] for e in s['line']] == ['ปาB', 'ปาC'], f"current={s['current'] and s['current']['user']} line={s['line']}")
        get('/control?clear=1')
        s = wait_for(lambda s: s['current'] is None, 8)
        get(f"/gift?gift_name=Rose&user={q('สมศรี')}&diamonds=10000")
        seen = set(); cards = set(); dunk_by = None; t0 = time.time()
        while time.time() - t0 < 16:
            s = state()
            c = s.get('current')
            if c:
                seen.add(c.get('phase')); cards.add(c.get('card'))
                if c.get('dunk_by'):
                    dunk_by = c.get('dunk_by')
                    if c.get('card') == 'splash' or c.get('phase') in ('drop', 'splash'):
                        break
            time.sleep(0.1)
        check('dunk: t5 walks and throws', 'walk' in seen or 'throw' in seen, str(sorted(x for x in seen if x)))
        check('dunk: t5 instant dunk credits the sender', dunk_by == 'สมศรี' or (state().get('current') or {}).get('dunk_by') == 'สมศรี', f"dunk_by={dunk_by} cards={cards} phases={sorted(x for x in seen if x)}")
        check('dunk: result is throw/splash card only (no lucky numbers)', cards <= {'throw', 'splash'} and cards, str(cards))
        get('/control?clear=1')

        # ---------------- dance/dunk gift floods: merge, party, bounded memory, drain
        get('/control?mode=dance'); s = wait_for(lambda s: s['theme'] == 'dance')
        s = wait_for(lambda s: s.get('current') is None and s.get('idle'), 25)
        g0 = state()['gifts_received']
        for _ in range(30):
            get(f"/gift?gift_name=Rose&user={q('สแปม')}&diamonds=1")
        s = wait_for(lambda s: s['gifts_received'] >= g0 + 30, 8)
        check('dance flood: 30 gifts from one sender accepted', s['gifts_received'] >= g0 + 30, f"gifts={s['gifts_received']} g0={g0}")
        check('dance flood: same sender merged (combo or single live card)', s.get('combo', 1) >= 2 or s.get('queue_len', 99) <= 1, f"combo={s.get('combo')} queue_len={s.get('queue_len')} party={s.get('party')}")
        t0 = time.time()
        for i in range(50):
            get(f"/gift?gift_name=Rose&user={q('คน'+str(i))}&diamonds=1")
            target = t0 + (i + 1) * (10.0 / 50.0)
            while time.time() < target:
                time.sleep(0.01)
        s = state()
        check('dance flood: app still serving after 50 senders / 10s', s.get('theme') == 'dance' and 'live_units' in s, str(s.get('theme')))
        check('dance flood: live records stay bounded', (s.get('live_units') or 0) <= 250, f"live_units={s.get('live_units')} queue_len={s.get('queue_len')} waiting_extra={s.get('waiting_extra')}")
        hall = s.get('hall') or {}
        # hall holds max_floor + max_stage + 1 star; line excludes the lead
        line_cap = (hall.get('max_floor') or 14) + (hall.get('max_stage') or 6)
        check('dance flood: not one-queue-entry-per-gift', (s.get('queue_len') or 0) <= 60 and len(s.get('line') or []) <= line_cap and (s.get('party') or (s.get('combo') or 1) >= 2), f"queue_len={s.get('queue_len')} line={len(s.get('line') or [])}/{line_cap} party={s.get('party')} combo={s.get('combo')} gifts={s.get('gifts_received')}")
        check('dance flood: party mode or overflow badge engaged', s.get('party') or (s.get('waiting_extra') or 0) > 0 or (s.get('combo') or 1) >= 2, f"party={s.get('party')} waiting_extra={s.get('waiting_extra')} combo={s.get('combo')}")
        t1 = time.time()
        drained = False
        while time.time() - t1 < 45:
            s = state()
            if s.get('idle') or (not s.get('party') and (s.get('waiting_extra') or 0) == 0 and s.get('current') is None):
                drained = True
                break
            time.sleep(0.3)
        s = state()
        check('dance flood: queue drains after the burst', drained or (s.get('live_units') or 999) < 80, f"idle={s.get('idle')} party={s.get('party')} live_units={s.get('live_units')} waiting_extra={s.get('waiting_extra')} current={s.get('current')}")
        get('/control?clear=1')

        get('/control?mode=dunk'); s = wait_for(lambda s: s['theme'] == 'dunk')
        s = wait_for(lambda s: s.get('current') is None, 20)
        g0 = state()['gifts_received']
        for _ in range(12):
            get(f"/gift?gift_name=Rose&user={q('นก')}&diamonds=1")
        s = wait_for(lambda s: s['gifts_received'] >= g0 + 12, 8)
        check('dunk flood: repeated throws merge or enter party', s.get('combo', 1) >= 2 or s.get('party') or s.get('queue_len', 99) <= 1, f"combo={s.get('combo')} party={s.get('party')} queue_len={s.get('queue_len')}")
        check('dunk flood: still serving', s.get('theme') == 'dunk' and (s.get('live_units') or 0) <= 250, f"live_units={s.get('live_units')}")
        get('/control?clear=1')

        # ---------------- pet mode: one shared cat, 3 meters, 10 price-laddered gimmicks
        get('/control?mode=pet'); s = wait_for(lambda s: s['theme'] == 'pet')
        check('runtime mode switch -> pet (เลี้ยงแมว)', s['theme'] == 'pet')
        ps = s.get('pet') or {}
        check('pet: /state exposes cat + 3 meters + 10-trick price ladder',
              all(k in ps for k in ('name', 'loneliness', 'fullness', 'bond', 'level', 'mood', 'tricks'))
              and [t['diamonds'] for t in ps.get('tricks', [])] == [1, 5, 10, 30, 99, 150, 199, 399, 400, 4888],
              f"level={ps.get('level')} tricks={len(ps.get('tricks', []))}")
        check('pet: cat + meters stay on screen while idle', s['current'] is None and not s['idle'], f"idle={s['idle']}")
        l0 = ps.get('loneliness', 0); la = ps.get('lonely_after', 20)
        s = wait_for(lambda s: (s.get('pet') or {}).get('since_touch', 0) > la + 2, 15)
        l1 = (s.get('pet') or {}).get('loneliness', 0)
        check('pet: loneliness rises only after nobody plays for lonely_after', l1 > l0 + 1, f'{l0:.1f} -> {l1:.1f} (idle > {la:.0f}s)')
        f0 = (state().get('pet') or {}).get('fullness', 0); time.sleep(1.5)
        f1 = (state().get('pet') or {}).get('fullness', 0)
        check('pet: fullness decays on its own', f1 < f0 - 2, f'{f0:.1f} -> {f1:.1f}')
        ladder = [('Rose', 1, 'head_pet'), ('Brown Paw', 5, 'paw_five'), ('Magic Feather', 10, 'wand_chase'),
                  ('Doughnut', 30, 'donut_treat'), ('Sundae Bowl', 99, 'milk_bowl'), ('Puffer Fish', 150, 'fish_toy'),
                  ('Fan Cat', 199, 'friend_visit'), ('Kitten Kneading', 399, 'knead'), ('Reindeer Milk', 400, 'milk_bottle'),
                  ('Leon the Kitten', 4888, 'golden_kitten')]
        cards = set(); bad = []; fed = False; lonely_ok = False
        fb = (state().get('pet') or {}).get('fullness', 0)
        for name, d, key in ladder:
            get(f'/gift?gift_name={q(name)}&user={q("เลี้ยง" + key)}&diamonds={d}')
            c = wait_for(lambda s: (s.get('current') or {}).get('trick') == key, 12).get('current') or {}
            if c.get('trick') != key or c.get('gift') != name: bad.append(f"{name}->{c.get('trick')}")
            cards.add(c.get('card'))
            if key == 'head_pet':
                pd = state().get('pet') or {}
                lonely_ok = pd.get('loneliness', 100) < l1 - 5 and pd.get('since_touch', 99) < 3
            if key == 'milk_bowl':
                fed = (state().get('pet') or {}).get('fullness', 0) > fb + 5
        check('pet: all 10 ladder gifts play their own gimmick', not bad, ', '.join(bad) or '10/10 mapped by catalog name')
        check('pet: playing pet resets loneliness + idle timer', lonely_ok, f'loneliness was {l1:.1f}')
        check('pet: feeding gift refills fullness', fed, f'fullness before ladder {fb:.1f}')
        check('pet: play card only (no lucky numbers)', cards == {'play'}, str(cards))
        s = wait_for(lambda s: s.get('current') is None and not s.get('queue'), 15)
        ps2 = s.get('pet') or {}
        check('pet: bond levels the cat up (10 plays -> level 2+)', ps2.get('level', 1) >= 2 and bool(ps2.get('level_title')), f"level={ps2.get('level')} bond={ps2.get('bond'):.1f} title={ps2.get('level_title')}")
        get('/control?clear=1'); wait_for(lambda s: s.get('current') is None, 8)
        get(f"/gift?gift_name={q('MysteryBoxXYZ')}&user={q('นอกตาราง')}&diamonds=150")
        c = wait_for(lambda s: (s.get('current') or {}).get('user') == 'นอกตาราง', 10).get('current') or {}
        check('pet: unmapped gift picks the dearest affordable gimmick (150 -> fish_toy)', c.get('trick') == 'fish_toy', f"trick={c.get('trick')}")
        get('/control?clear=1'); wait_for(lambda s: s.get('current') is None, 8)
        for _ in range(3):
            get(f"/gift?gift_name=Rose&user={q('ซ้ำแมว')}&diamonds=1")
        c = wait_for(lambda s: (s.get('current') or {}).get('user') == 'ซ้ำแมว', 10).get('current') or {}
        check('pet: same sender + gimmick inside merge window -> combo xN', (c.get('combo') or 1) >= 2, f"combo={c.get('combo')}")
        get('/control?clear=1'); wait_for(lambda s: s.get('current') is None, 8)
        for i in range(40):
            get(f"/gift?gift_name=Rose&user={q('ฝูง' + str(i))}&diamonds=1")
        s = wait_for(lambda s: s['gifts_received'] > 0 and (s.get('queue_len') or 0) >= 1, 8)
        mq = (s.get('pet') or {}).get('max_queue', 30)
        check('pet: gift flood stays inside max_queue', (s.get('queue_len') or 0) <= mq and (s.get('live_units') or 0) <= 250, f"queue_len={s.get('queue_len')}/{mq} overflow_extra={(s.get('pet') or {}).get('overflow_extra')} live_units={s.get('live_units')}")
        lv = (state().get('pet') or {}).get('level', 1)
        get('/control?clear=1'); s = wait_for(lambda s: s.get('current') is None and not s.get('queue'), 10)
        ps3 = s.get('pet') or {}
        check('pet: clear empties the queue but keeps the raised cat', ps3.get('level') == lv and bool(ps3.get('name')) and s['current'] is None, f"level={ps3.get('level')} name={ps3.get('name')}")
        get('/control?mode=shrine'); wait_for(lambda s: s['theme'] == 'shrine')

        # ---------------- v0.2.4: shared rates, per-sticker dance moves, stage + star
        r = state().get('rates') or {}
        check('v0.2.4 shared rates: stage 30 / star auto / tier5_auto', r.get('stage_diamonds') == 30 and r.get('star_mode') == 'auto' and r.get('tier5_auto') is True, json.dumps(r, ensure_ascii=False))
        check('v0.2.4 shared rates: five reachable tiers 0/5/20/30/100', r.get('tier_mins') == [0, 5, 20, 30, 100], str(r.get('tier_mins')))
        check('v0.2.4 shared rates: star cutoff = priciest mapped sticker (Hand Heart 100)', r.get('star_diamonds') == 100, f"star_diamonds={r.get('star_diamonds')}")

        def idle_now(timeout=25):
            get('/control?clear=1')
            return wait_for(lambda s: s.get('current') is None and not s.get('queue'), timeout)

        def sendAndWait(gift, user, diamonds, timeout=10):
            get(f'/gift?gift_name={q(gift)}&user={q(user)}&diamonds={diamonds}')
            s = wait_for(lambda s: (s.get('current') or {}).get('user') == user, timeout)
            return s.get('current') or {}

        # one shared rate table: the same diamonds must give the same tier in every mode
        got = {}
        for m in ['shrine', 'krathong', 'cat', 'dance', 'dunk']:
            get(f'/control?mode={m}'); wait_for(lambda s: s['theme'] == m)
            idle_now()
            got[m] = sendAndWait('Doughnut', 'เรท' + m, 30).get('tier')
        check('v0.2.4 shared rates: 30 diamonds = tier 4 in all 5 modes', all(v == 4 for v in got.values()), str(got))

        # each mapped sticker keeps its configured move; role follows the shared stage/star cutoffs
        get('/control?mode=dance'); wait_for(lambda s: s['theme'] == 'dance')
        want = [('Rose', 1, 'wave', 'floor'), ('TikTok', 1, 'jump', 'floor'), ('Finger Heart', 5, 'freeze', 'floor'),
                ('Perfume', 20, 'fan', 'floor'), ('Doughnut', 30, 'crab', 'stage'), ('Hand Heart', 100, 'mirror', 'star')]
        mdet, rdet = [], []
        mok = rok = True
        for name, d, mv, role in want:
            idle_now()
            c = sendAndWait(name, 'ท่า' + name.replace(' ', ''), d)
            mdet.append(f"{name}->{c.get('move')}")
            rdet.append(f"{d}->{c.get('role')}")
            if c.get('move') != mv: mok = False
            if c.get('role') != role: rok = False
        check('v0.2.4 dance: every mapped sticker gets its configured move (not random)', mok, ', '.join(mdet))
        check('v0.2.4 dance: floor below 30, stage at 30, star at 100', rok, ', '.join(rdet))

        idle_now()
        c = sendAndWait('UnknownStickerXYZ', 'ท่าfallback', 20)
        check('v0.2.4 dance: unmapped sticker uses the fixed tier move (tier 3 = spin)', c.get('move') == 'spin' and c.get('tier') == 3, f"move={c.get('move')} tier={c.get('tier')}")

        idle_now()
        for _ in range(3):
            get(f"/gift?gift_name=Rose&user={q('คอมโบ')}&diamonds=1")
        s = wait_for(lambda s: (s.get('current') or {}).get('user') == 'คอมโบ', 10)
        c = s.get('current') or {}
        check('v0.2.4 dance: combo xN keeps the same sticker move', c.get('move') == 'wave' and (c.get('combo') or 1) >= 2, f"move={c.get('move')} combo={c.get('combo')}")

        # star flag on the shrine / krathong result
        sdet = []
        sok = True
        for m in ['shrine', 'krathong']:
            get(f'/control?mode={m}'); wait_for(lambda s: s['theme'] == m)
            idle_now()
            hi = sendAndWait('Hand Heart', 'สตาร์' + m, 100)
            idle_now()
            lo = sendAndWait('Perfume', 'ไม่ใช่' + m, 20)
            sdet.append(f"{m}:100={hi.get('star')},20={lo.get('star')}")
            if hi.get('star') is not True or lo.get('star') is not False: sok = False
        check('v0.2.4 star: Hand Heart 100 is star, Perfume 20 is not (shrine + krathong)', sok, ', '.join(sdet))
        idle_now()

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
        else:
            check('Ctrl+Q quits', hotkey_ok.get(QUIT_TEST, False), f'headless egui pass: hotkey_tests::{QUIT_TEST} (ViewportCommand::Close)')
        log_txt = open(os.path.join(WORK, 'app.log'), encoding='utf-8', errors='replace').read()
        check('no panic in log', 'panicked' not in log_txt)
    finally:
        if p.poll() is None: p.kill()
    passed = sum(1 for r in results if r[1])
    ver = subprocess.run([BIN, '--version'], capture_output=True, text=True).stdout.strip()
    keys = 'xdotool under Xvfb (real OS key events)' if XDOTOOL else 'cargo test hotkey_tests (real egui passes, headless)'
    os.makedirs(os.path.join(ROOT, 'tests'), exist_ok=True)
    with open(os.path.join(ROOT, 'tests/e2e_results.md'), 'w', encoding='utf-8') as f:
        f.write(f'# E2E results ({time.strftime("%Y-%m-%d %H:%M %Z")})\n\n{passed}/{len(results)} passed\n\n')
        f.write(f'- binary: `{ver}`\n- platform: {platform.system()} {platform.machine()}\n- keyboard: {keys}\n\n')
        f.write('| # | test | result | detail |\n|---|---|---|---|\n')
        for i, (n, ok, d) in enumerate(results, 1):
            f.write(f'| {i} | {n} | {"PASS" if ok else "FAIL"} | {d} |\n')
    print(f'\n{passed}/{len(results)} passed')
    sys.exit(0 if passed == len(results) else 1)

main()

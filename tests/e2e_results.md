# E2E results (2026-11-14 14:21 +07)

111/111 passed

- binary: `tiktok-shrine-live 0.2.5`
- platform: Darwin arm64
- keyboard: cargo test hotkey_tests (real egui passes, headless)

| # | test | result | detail |
|---|---|---|---|
| 1 | app starts, /state responds | PASS | version 0.2.5 |
| 2 | GET /health | PASS |  |
| 3 | update state: one-button fields (pending/runtime/connector) | PASS | {"pending": [], "runtime_current": null, "bridge_current": "2.5.0"} |
| 4 | --which-node resolves a runtime (bundled/system) | PASS | system /opt/homebrew/bin/node v26.11.1 |
| 5 | idle at start (still picture, no character) | PASS |  |
| 6 | still idle: scene frozen (no moving effects) | PASS | 0.000 -> 0.000 |
| 7 | key 1 -> pray test tier | PASS | headless egui pass: hotkey_tests::key_1_enqueues_pray_at_test_tier |
| 8 | key 2 -> siamsi test tier | PASS | headless egui pass: hotkey_tests::key_2_enqueues_siamsi_at_test_tier |
| 9 | key Q -> pray t1 | PASS | headless egui pass: hotkey_tests::pray_tier_keys_q_w_e_r_t_map_to_tiers_1_5 |
| 10 | key W -> pray t2 | PASS | headless egui pass: hotkey_tests::pray_tier_keys_q_w_e_r_t_map_to_tiers_1_5 |
| 11 | key E -> pray t3 | PASS | headless egui pass: hotkey_tests::pray_tier_keys_q_w_e_r_t_map_to_tiers_1_5 |
| 12 | key R -> pray t4 | PASS | headless egui pass: hotkey_tests::pray_tier_keys_q_w_e_r_t_map_to_tiers_1_5 |
| 13 | key T -> pray t5 | PASS | headless egui pass: hotkey_tests::pray_tier_keys_q_w_e_r_t_map_to_tiers_1_5 |
| 14 | key A -> siamsi t1 | PASS | headless egui pass: hotkey_tests::siamsi_tier_keys_a_s_d_f_g_map_to_tiers_1_5 |
| 15 | key S -> siamsi t2 | PASS | headless egui pass: hotkey_tests::siamsi_tier_keys_a_s_d_f_g_map_to_tiers_1_5 |
| 16 | key D -> siamsi t3 | PASS | headless egui pass: hotkey_tests::siamsi_tier_keys_a_s_d_f_g_map_to_tiers_1_5 |
| 17 | key F -> siamsi t4 | PASS | headless egui pass: hotkey_tests::siamsi_tier_keys_a_s_d_f_g_map_to_tiers_1_5 |
| 18 | key G -> siamsi t5 | PASS | headless egui pass: hotkey_tests::siamsi_tier_keys_a_s_d_f_g_map_to_tiers_1_5 |
| 19 | Q W E R T map to tiers 1..5 | PASS | headless egui pass: hotkey_tests::pray_tier_keys_q_w_e_r_t_map_to_tiers_1_5 |
| 20 | A S D F G map to tiers 1..5 | PASS | headless egui pass: hotkey_tests::siamsi_tier_keys_a_s_d_f_g_map_to_tiers_1_5 |
| 21 | Shift+3 selects test tier 3 | PASS | headless egui pass: hotkey_tests::shift_3_selects_test_tier_3 |
| 22 | Delete clears the queue | PASS | headless egui pass: hotkey_tests::delete_clears_the_queue |
| 23 | F8 toggles click-through | PASS | headless egui pass: hotkey_tests::f8_toggles_click_through |
| 24 | F9 cycles background | PASS | headless egui pass: hotkey_tests::f9_cycles_background_transparent_green_scene |
| 25 | F2 switches mode | PASS | headless egui pass: hotkey_tests::f2_switches_mode |
| 26 | F2 cycles all 6 modes (shrine -> krathong -> cat -> dance -> dunk -> pet) | PASS | headless egui pass: hotkey_tests::f2_cycles_all_6_modes_in_order |
| 27 | F1 opens/closes settings | PASS | headless egui pass: hotkey_tests::f1_opens_and_closes_settings |
| 28 | GET /event?action=pray | PASS |  |
| 29 | GET /gift TikTok -> siamsi (gift mapping) | PASS | {'action': 'Siamsi', 'tier': 1, 'user': 'nana'} |
| 30 | gift id 5655 -> pray (mapping by id) | PASS |  |
| 31 | POST /event JSON | PASS |  |
| 32 | POST /webhook (TikFinity style JSON) | PASS |  |
| 33 | unknown gift -> default action (pray) | PASS |  |
| 34 | 5 tier thresholds incl. diamonds x repeat (0/5/20/30/100) | PASS | 1x1->1, 4x1->1, 5x1->2, 19x1->2, 20x1->3, 29x1->3, 30x1->4, 99x1->4, 100x1->5, 1000x1->5, 1x5->2, 5x4->3, 1x30->4, 1x100->5, 500x20->5 |
| 35 | combo streak (repeat=5) plays once with summed value (max_plays_per_gift=1) | PASS | [{'action': 'Pray', 'tier': 2, 'user': 'combo'}] |
| 36 | 10 rapid gifts all queued in order | PASS | queue_len=10 ['fan0', 'fan1', 'fan2', 'fan3', 'fan4', 'fan5', 'fan6', 'fan7', 'fan8', 'fan9'] |
| 37 | 3 senders: front one plays, other two visible in line | PASS | current=ผู้ส่งA line=[{'user': 'ผู้ส่งB', 'x': 430.0}, {'user': 'ผู้ส่งC', 'x': 285.0}] |
| 38 | 3 senders: line is single-file, no overlap (>=100 px apart, on screen) | PASS | x=[430.0, 285.0] |
| 39 | emoji stripped from name label (no tofu) | PASS |  |
| 40 | shrine result card matches action (pray -> lucky numbers, siamsi -> fortune) | PASS | Pray -> numbers |
| 41 | 3 senders: line advances (2nd sender steps up, 3rd moves forward) | PASS | line=[{'user': 'ผู้ส่งC', 'x': 317.0}] |
| 42 | 3 senders: each gets their own turn in order | PASS |  |
| 43 | layout toggle full/top (window resizes) | PASS | [540.0, 960.0] vs [540.0, 480.0] |
| 44 | background mode green | PASS |  |
| 45 | background mode scene | PASS |  |
| 46 | background mode transparent | PASS |  |
| 47 | transparent idle: ground props hidden | PASS | props_alpha=0.0 |
| 48 | transparent show: props fade in | PASS | props_alpha=1.0 |
| 49 | runtime mode switch -> krathong | PASS |  |
| 50 | krathong: krathong is released and keeps floating | PASS | floating=1 |
| 51 | runtime mode switch -> cat (ให้อาหารแมว) | PASS |  |
| 52 | cat: 3 senders -> front cat + 2 cats visible in line | PASS | current=แมวA line=[{'user': 'แมวB', 'x': -76.0}, {'user': 'แมวC', 'x': -76.0}] |
| 53 | cat: walk -> pour -> eat -> card phases | PASS | ['card', 'eat', 'leave', 'pour', 'walk'] |
| 54 | cat: result is thank-you card only (no lucky numbers) | PASS | {'thanks'} |
| 55 | cat: line advances to next sender | PASS | line=[{'user': 'แมวC', 'x': 146.0}] |
| 56 | runtime mode switch -> dance (โหมดเต้น) | PASS |  |
| 57 | dance: 3 senders -> lead + 2 on the floor | PASS | current=เต้นA line=[{'user': 'เต้นB', 'x': -80.0}, {'user': 'เต้นC', 'x': -80.0}] |
| 58 | dance: walk then dance phases | PASS | ['dance', 'leave'] |
| 59 | dance: result is challenge card only (no lucky numbers) | PASS | {'challenge'} |
| 60 | dance: lead advances B -> C then clears (hall dancers leave on their own clock) | PASS | succession=[('เต้นB', ['เต้นC']), ('เต้นC', []), (None, [])] |
| 61 | runtime mode switch -> dunk (ปาตกน้ำ) | PASS |  |
| 62 | dunk: 3 senders -> thrower + 2 waiting | PASS | current=ปาA line=[{'user': 'ปาB', 'x': -80.0}, {'user': 'ปาC', 'x': -80.0}] |
| 63 | dunk: t5 walks and throws | PASS | ['drop', 'throw', 'walk'] |
| 64 | dunk: t5 instant dunk credits the sender | PASS | dunk_by=สมศรี cards={'throw', 'splash'} phases=['drop', 'throw', 'walk'] |
| 65 | dunk: result is throw/splash card only (no lucky numbers) | PASS | {'throw', 'splash'} |
| 66 | dance flood: 30 gifts from one sender accepted | PASS | gifts=77 g0=47 |
| 67 | dance flood: same sender merged (combo or single live card) | PASS | combo=30 queue_len=0 party=True |
| 68 | dance flood: app still serving after 50 senders / 10s | PASS | dance |
| 69 | dance flood: live records stay bounded | PASS | live_units=105 queue_len=50 waiting_extra=37 |
| 70 | dance flood: not one-queue-entry-per-gift | PASS | queue_len=50 line=13/20 party=True combo=30 gifts=127 |
| 71 | dance flood: party mode or overflow badge engaged | PASS | party=True waiting_extra=37 combo=30 |
| 72 | dance flood: queue drains after the burst | PASS | idle=True party=False live_units=6 waiting_extra=0 current=None |
| 73 | dunk flood: repeated throws merge or enter party | PASS | combo=12 party=True queue_len=0 |
| 74 | dunk flood: still serving | PASS | live_units=14 |
| 75 | runtime mode switch -> pet (เลี้ยงแมว) | PASS |  |
| 76 | pet: /state exposes cat + 3 meters + 10-trick price ladder | PASS | level=1 tricks=10 |
| 77 | pet: cat + meters stay on screen while idle | PASS | idle=False |
| 78 | pet: loneliness rises only after nobody plays for lonely_after | PASS | 12.0 -> 15.3 (idle > 20s) |
| 79 | pet: cat roams the room on its own (poses / walking) | PASS | acts=['loaf', 'sit', 'sniff', 'stretch', 'walk'] x_span=515 |
| 80 | pet: fullness decays on its own | PASS | 3.4 -> 0.0 |
| 81 | pet: all 10 ladder gifts play their own gimmick | PASS | 10/10 mapped by catalog name |
| 82 | pet: playing pet resets loneliness + idle timer | PASS | loneliness was 100.0 |
| 83 | pet: feeding gift refills fullness | PASS | fullness before ladder 0.0 |
| 84 | pet: play card only (no lucky numbers) | PASS | {'play'} |
| 85 | pet: bond levels the cat up (10 plays -> level 2+) | PASS | level=2 bond=5.0 title=แมวขี้อ้อน |
| 86 | pet: unmapped gift picks the dearest affordable gimmick (150 -> fish_toy) | PASS | trick=fish_toy |
| 87 | pet: same sender + gimmick inside merge window -> combo xN | PASS | combo=3 |
| 88 | pet: gift flood stays inside max_queue | PASS | queue_len=30/30 overflow_extra=9 live_units=41 |
| 89 | pet: clear empties the queue but keeps the raised cat | PASS | level=2 name=น้องมอมแมม |
| 90 | v0.2.4 shared rates: stage 30 / star auto / tier5_auto | PASS | {"stage_diamonds": 30, "star_diamonds": 100, "star_mode": "auto", "tier5_auto": true, "tier_mins": [0, 5, 20, 30, 100]} |
| 91 | v0.2.4 shared rates: five reachable tiers 0/5/20/30/100 | PASS | [0, 5, 20, 30, 100] |
| 92 | v0.2.4 shared rates: star cutoff = priciest mapped sticker (Hand Heart 100) | PASS | star_diamonds=100 |
| 93 | v0.2.4 shared rates: 30 diamonds = tier 4 in all 5 modes | PASS | {'shrine': 4, 'krathong': 4, 'cat': 4, 'dance': 4, 'dunk': 4} |
| 94 | v0.2.4 dance: every mapped sticker gets its configured move (not random) | PASS | Rose->wave, TikTok->jump, Finger Heart->freeze, Perfume->fan, Doughnut->crab, Hand Heart->mirror |
| 95 | v0.2.4 dance: floor below 30, stage at 30, star at 100 | PASS | 1->floor, 1->floor, 5->floor, 20->floor, 30->stage, 100->star |
| 96 | v0.2.4 dance: unmapped sticker uses the fixed tier move (tier 3 = spin) | PASS | move=spin tier=3 |
| 97 | v0.2.4 dance: combo xN keeps the same sticker move | PASS | move=wave combo=3 |
| 98 | v0.2.4 star: Hand Heart 100 is star, Perfume 20 is not (shrine + krathong) | PASS | shrine:100=True,20=False, krathong:100=True,20=False |
| 99 | runtime mode switch -> shrine | PASS |  |
| 100 | click-through on/off via /control | PASS |  |
| 101 | stop: gifts ignored while stopped | PASS |  |
| 102 | start: gifts accepted again | PASS |  |
| 103 | idle=hidden: scene fades fully transparent when idle | PASS | alpha=0.00 |
| 104 | idle=hidden: gift fades the scene back in and plays | PASS |  |
| 105 | idle=ambient: subtle motion keeps running | PASS |  |
| 106 | idle=still: back to frozen still picture | PASS |  |
| 107 | save writes config.toml | PASS |  |
| 108 | reload picks up edited config.toml | PASS |  |
| 109 | saved config keeps Thai text (UTF-8) | PASS |  |
| 110 | Ctrl+Q quits | PASS | headless egui pass: hotkey_tests::cmd_q_requests_window_close (ViewportCommand::Close) |
| 111 | no panic in log | PASS |  |

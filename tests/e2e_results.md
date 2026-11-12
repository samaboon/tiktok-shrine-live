# E2E results (2026-11-13 01:21 +07)

78/78 passed

| # | test | result | detail |
|---|---|---|---|
| 1 | app starts, /state responds | PASS | version 0.2.2 |
| 2 | GET /health | PASS |  |
| 3 | update state: one-button fields (pending/runtime/connector) | PASS | {"pending": [], "runtime_current": null, "bridge_current": "2.5.0"} |
| 4 | --which-node resolves a runtime (bundled/system) | PASS | system /usr/local/bin/node v24.22.1 |
| 5 | idle at start (still picture, no character) | PASS |  |
| 6 | still idle: scene frozen (no moving effects) | PASS | 0.000 -> 0.000 |
| 7 | key 1 -> pray test tier | PASS |  |
| 8 | key 2 -> siamsi test tier | PASS |  |
| 9 | key Q -> pray t1 | PASS |  |
| 10 | key W -> pray t2 | PASS |  |
| 11 | key E -> pray t3 | PASS |  |
| 12 | key R -> pray t4 | PASS |  |
| 13 | key T -> pray t5 | PASS |  |
| 14 | key A -> siamsi t1 | PASS |  |
| 15 | key S -> siamsi t2 | PASS |  |
| 16 | key D -> siamsi t3 | PASS |  |
| 17 | key F -> siamsi t4 | PASS |  |
| 18 | key G -> siamsi t5 | PASS |  |
| 19 | Q W E R T map to tiers 1..5 | PASS | [1, 2, 3, 4, 5] |
| 20 | A S D F G map to tiers 1..5 | PASS | [1, 2, 3, 4, 5] |
| 21 | Shift+3 selects test tier 3 | PASS |  |
| 22 | Delete clears the queue | PASS |  |
| 23 | F8 toggles click-through | PASS |  |
| 24 | F9 cycles background | PASS | transparent -> green |
| 25 | F2 switches mode | PASS | shrine -> krathong |
| 26 | F2 cycles all 5 modes (shrine -> krathong -> cat -> dance -> dunk) | PASS | shrine -> krathong -> cat -> dance -> dunk |
| 27 | F1 opens/closes settings | PASS |  |
| 28 | GET /event?action=pray | PASS |  |
| 29 | GET /gift TikTok -> siamsi (gift mapping) | PASS | {'action': 'Siamsi', 'tier': 1, 'user': 'nana'} |
| 30 | gift id 5655 -> pray (mapping by id) | PASS |  |
| 31 | POST /event JSON | PASS |  |
| 32 | POST /webhook (TikFinity style JSON) | PASS |  |
| 33 | unknown gift -> default action (pray) | PASS |  |
| 34 | 5 tier thresholds incl. diamonds x repeat (0/10/100/1000/10000) | PASS | 1x1->1, 9x1->1, 10x1->2, 99x1->2, 100x1->3, 999x1->3, 1000x1->4, 9999x1->4, 10000x1->5, 1x10->2, 5x20->3, 1x1000->4, 500x20->5 |
| 35 | combo streak (repeat=5) plays once with summed value (max_plays_per_gift=1) | PASS | [{'action': 'Pray', 'tier': 1, 'user': 'combo'}] |
| 36 | 10 rapid gifts all queued in order | PASS | queue_len=10 ['fan0', 'fan1', 'fan2', 'fan3', 'fan4', 'fan5', 'fan6', 'fan7', 'fan8', 'fan9'] |
| 37 | 3 senders: front one plays, other two visible in line | PASS | current=ผู้ส่งA line=[{'user': 'ผู้ส่งB', 'x': 430.0}, {'user': 'ผู้ส่งC', 'x': 285.0}] |
| 38 | 3 senders: line is single-file, no overlap (>=100 px apart, on screen) | PASS | x=[430.0, 285.0] |
| 39 | emoji stripped from name label (no tofu) | PASS |  |
| 40 | shrine result card matches action (pray -> lucky numbers, siamsi -> fortune) | PASS | Pray -> numbers |
| 41 | 3 senders: line advances (2nd sender steps up, 3rd moves forward) | PASS | line=[{'user': 'ผู้ส่งC', 'x': 381.0}] |
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
| 52 | cat: 3 senders -> front cat + 2 cats visible in line | PASS | current=แมวA line=[{'user': 'แมวB', 'x': -49.0}, {'user': 'แมวC', 'x': -49.0}] |
| 53 | cat: walk -> pour -> eat -> card phases | PASS | ['card', 'eat', 'leave', 'pour', 'walk'] |
| 54 | cat: result is thank-you card only (no lucky numbers) | PASS | {'thanks'} |
| 55 | cat: line advances to next sender | PASS | line=[{'user': 'แมวC', 'x': 419.0}] |
| 56 | runtime mode switch -> dance (โหมดเต้น) | PASS |  |
| 57 | dance: 3 senders -> lead + 2 on the floor | PASS | current=เต้นA line=[{'user': 'เต้นB', 'x': -80.0}, {'user': 'เต้นC', 'x': -80.0}] |
| 58 | dance: walk then dance phases | PASS | ['dance', 'leave', 'walk'] |
| 59 | dance: result is challenge card only (no lucky numbers) | PASS | {'challenge'} |
| 60 | dance: line advances to next sender | PASS | line=[{'user': 'เต้นC', 'x': -80.0}] |
| 61 | runtime mode switch -> dunk (ปาตกน้ำ) | PASS |  |
| 62 | dunk: 3 senders -> thrower + 2 waiting | PASS | current=ปาA line=[{'user': 'ปาB', 'x': -80.0}, {'user': 'ปาC', 'x': -80.0}] |
| 63 | dunk: t5 walks and throws | PASS | ['drop', 'throw', 'walk'] |
| 64 | dunk: t5 instant dunk credits the sender | PASS | dunk_by=สมศรี cards={'splash', 'throw'} phases=['drop', 'throw', 'walk'] |
| 65 | dunk: result is throw/splash card only (no lucky numbers) | PASS | {'splash', 'throw'} |
| 66 | runtime mode switch -> shrine | PASS |  |
| 67 | click-through on/off via /control | PASS |  |
| 68 | stop: gifts ignored while stopped | PASS |  |
| 69 | start: gifts accepted again | PASS |  |
| 70 | idle=hidden: scene fades fully transparent when idle | PASS | alpha=0.00 |
| 71 | idle=hidden: gift fades the scene back in and plays | PASS |  |
| 72 | idle=ambient: subtle motion keeps running | PASS |  |
| 73 | idle=still: back to frozen still picture | PASS |  |
| 74 | save writes config.toml | PASS |  |
| 75 | reload picks up edited config.toml | PASS |  |
| 76 | saved config keeps Thai text (UTF-8) | PASS |  |
| 77 | Ctrl+Q quits | PASS |  |
| 78 | no panic in log | PASS |  |

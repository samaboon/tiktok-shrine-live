# E2E results (2026-11-12 22:54 +07)

64/64 passed

| # | test | result | detail |
|---|---|---|---|
| 1 | app starts, /state responds | PASS | version 0.2.1 |
| 2 | GET /health | PASS |  |
| 3 | idle at start (still picture, no character) | PASS |  |
| 4 | still idle: scene frozen (no moving effects) | PASS | 0.000 -> 0.000 |
| 5 | key 1 -> pray test tier | PASS |  |
| 6 | key 2 -> siamsi test tier | PASS |  |
| 7 | key Q -> pray t1 | PASS |  |
| 8 | key W -> pray t2 | PASS |  |
| 9 | key E -> pray t3 | PASS |  |
| 10 | key R -> pray t4 | PASS |  |
| 11 | key T -> pray t5 | PASS |  |
| 12 | key A -> siamsi t1 | PASS |  |
| 13 | key S -> siamsi t2 | PASS |  |
| 14 | key D -> siamsi t3 | PASS |  |
| 15 | key F -> siamsi t4 | PASS |  |
| 16 | key G -> siamsi t5 | PASS |  |
| 17 | Q W E R T map to tiers 1..5 | PASS | [1, 2, 3, 4, 5] |
| 18 | A S D F G map to tiers 1..5 | PASS | [1, 2, 3, 4, 5] |
| 19 | Shift+3 selects test tier 3 | PASS |  |
| 20 | Delete clears the queue | PASS |  |
| 21 | F8 toggles click-through | PASS |  |
| 22 | F9 cycles background | PASS | transparent -> green |
| 23 | F2 switches mode | PASS | shrine -> krathong |
| 24 | F2 cycles all 3 modes (shrine -> krathong -> cat) | PASS | shrine -> krathong -> cat |
| 25 | F1 opens/closes settings | PASS |  |
| 26 | GET /event?action=pray | PASS |  |
| 27 | GET /gift TikTok -> siamsi (gift mapping) | PASS | {'action': 'Siamsi', 'tier': 1, 'user': 'nana'} |
| 28 | gift id 5655 -> pray (mapping by id) | PASS |  |
| 29 | POST /event JSON | PASS |  |
| 30 | POST /webhook (TikFinity style JSON) | PASS |  |
| 31 | unknown gift -> default action (pray) | PASS |  |
| 32 | 5 tier thresholds incl. diamonds x repeat (0/10/100/1000/10000) | PASS | 1x1->1, 9x1->1, 10x1->2, 99x1->2, 100x1->3, 999x1->3, 1000x1->4, 9999x1->4, 10000x1->5, 1x10->2, 5x20->3, 1x1000->4, 500x20->5 |
| 33 | combo streak (repeat=5) plays once with summed value (max_plays_per_gift=1) | PASS | [{'action': 'Pray', 'tier': 1, 'user': 'combo'}] |
| 34 | 10 rapid gifts all queued in order | PASS | queue_len=10 ['fan0', 'fan1', 'fan2', 'fan3', 'fan4', 'fan5', 'fan6', 'fan7', 'fan8', 'fan9'] |
| 35 | 3 senders: front one plays, other two visible in line | PASS | current=ผู้ส่งA line=[{'user': 'ผู้ส่งB', 'x': 430.0}, {'user': 'ผู้ส่งC', 'x': 285.0}] |
| 36 | 3 senders: line is single-file, no overlap (>=100 px apart, on screen) | PASS | x=[430.0, 285.0] |
| 37 | emoji stripped from name label (no tofu) | PASS |  |
| 38 | shrine result card matches action (pray -> lucky numbers, siamsi -> fortune) | PASS | Pray -> numbers |
| 39 | 3 senders: line advances (2nd sender steps up, 3rd moves forward) | PASS | line=[{'user': 'ผู้ส่งC', 'x': 414.0}] |
| 40 | 3 senders: each gets their own turn in order | PASS |  |
| 41 | layout toggle full/top (window resizes) | PASS | [540.0, 960.0] vs [540.0, 480.0] |
| 42 | background mode green | PASS |  |
| 43 | background mode scene | PASS |  |
| 44 | background mode transparent | PASS |  |
| 45 | runtime mode switch -> krathong | PASS |  |
| 46 | krathong: krathong is released and keeps floating | PASS | floating=1 |
| 47 | runtime mode switch -> cat (ให้อาหารแมว) | PASS |  |
| 48 | cat: 3 senders -> front cat + 2 cats visible in line | PASS | current=แมวA line=[{'user': 'แมวB', 'x': -46.0}, {'user': 'แมวC', 'x': -46.0}] |
| 49 | cat: walk -> pour -> eat -> card phases | PASS | ['card', 'eat', 'leave', 'pour', 'walk'] |
| 50 | cat: result is thank-you card only (no lucky numbers) | PASS | {'thanks'} |
| 51 | cat: line advances to next sender | PASS | line=[{'user': 'แมวC', 'x': 369.0}] |
| 52 | runtime mode switch -> shrine | PASS |  |
| 53 | click-through on/off via /control | PASS |  |
| 54 | stop: gifts ignored while stopped | PASS |  |
| 55 | start: gifts accepted again | PASS |  |
| 56 | idle=hidden: scene fades fully transparent when idle | PASS | alpha=0.00 |
| 57 | idle=hidden: gift fades the scene back in and plays | PASS |  |
| 58 | idle=ambient: subtle motion keeps running | PASS |  |
| 59 | idle=still: back to frozen still picture | PASS |  |
| 60 | save writes config.toml | PASS |  |
| 61 | reload picks up edited config.toml | PASS |  |
| 62 | saved config keeps Thai text (UTF-8) | PASS |  |
| 63 | Ctrl+Q quits | PASS |  |
| 64 | no panic in log | PASS |  |

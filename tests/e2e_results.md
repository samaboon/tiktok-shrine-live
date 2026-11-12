# E2E results (2026-11-12 21:42 +07)

52/52 passed

| # | test | result | detail |
|---|---|---|---|
| 1 | app starts, /state responds | PASS | version 0.2.0 |
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
| 24 | F1 opens/closes settings | PASS |  |
| 25 | GET /event?action=pray | PASS |  |
| 26 | GET /gift TikTok -> siamsi (gift mapping) | PASS | {'action': 'Siamsi', 'tier': 1, 'user': 'nana'} |
| 27 | gift id 5655 -> pray (mapping by id) | PASS |  |
| 28 | POST /event JSON | PASS |  |
| 29 | POST /webhook (TikFinity style JSON) | PASS |  |
| 30 | unknown gift -> default action (pray) | PASS |  |
| 31 | 5 tier thresholds incl. diamonds x repeat (0/10/100/1000/10000) | PASS | 1x1->1, 9x1->1, 10x1->2, 99x1->2, 100x1->3, 999x1->3, 1000x1->4, 9999x1->4, 10000x1->5, 1x10->2, 5x20->3, 1x1000->4, 500x20->5 |
| 32 | combo streak (repeat=5) plays once with summed value (max_plays_per_gift=1) | PASS | [{'action': 'Pray', 'tier': 1, 'user': 'combo'}] |
| 33 | 10 rapid gifts all queued in order | PASS | queue_len=10 ['fan0', 'fan1', 'fan2', 'fan3', 'fan4', 'fan5', 'fan6', 'fan7', 'fan8', 'fan9'] |
| 34 | layout toggle full/top (window resizes) | PASS | [540.0, 960.0] vs [540.0, 480.0] |
| 35 | background mode green | PASS |  |
| 36 | background mode scene | PASS |  |
| 37 | background mode transparent | PASS |  |
| 38 | runtime mode switch -> krathong | PASS |  |
| 39 | krathong: krathong is released and keeps floating | PASS | floating=1 |
| 40 | runtime mode switch -> shrine | PASS |  |
| 41 | click-through on/off via /control | PASS |  |
| 42 | stop: gifts ignored while stopped | PASS |  |
| 43 | start: gifts accepted again | PASS |  |
| 44 | idle=hidden: scene fades fully transparent when idle | PASS | alpha=0.00 |
| 45 | idle=hidden: gift fades the scene back in and plays | PASS |  |
| 46 | idle=ambient: subtle motion keeps running | PASS |  |
| 47 | idle=still: back to frozen still picture | PASS |  |
| 48 | save writes config.toml | PASS |  |
| 49 | reload picks up edited config.toml | PASS |  |
| 50 | saved config keeps Thai text (UTF-8) | PASS |  |
| 51 | Ctrl+Q quits | PASS |  |
| 52 | no panic in log | PASS |  |

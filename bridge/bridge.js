// TikTok LIVE -> stdout JSON-lines bridge for tiktok-shrine-live
// ใช้ไลบรารี tiktok-live-connector (ไม่เป็นทางการ / reverse-engineered)
// Usage: node bridge.js <tiktok_username> [--sign-key KEY] [--post http://127.0.0.1:8787/event]
import { TikTokLiveConnection, WebcastEvent, ControlEvent } from 'tiktok-live-connector';

const args = process.argv.slice(2);
const username = (args.find(a => !a.startsWith('--')) || '').replace(/^@/, '');
const opt = (name) => { const i = args.indexOf(name); return i >= 0 ? args[i + 1] : undefined; };
const signKey = opt('--sign-key') || process.env.SIGN_API_KEY || undefined;
const postUrl = opt('--post');

if (!username) { console.error('usage: node bridge.js <username> [--sign-key KEY] [--post URL]'); process.exit(2); }

function emit(obj) {
  const line = JSON.stringify(obj);
  console.log(line);
  if (postUrl && obj.type === 'gift') {
    fetch(postUrl, { method: 'POST', headers: { 'content-type': 'application/json' }, body: line })
      .catch(e => console.error('[bridge] post failed:', e.message));
  }
}

const conn = new TikTokLiveConnection(username, {
  signApiKey: signKey,
  enableExtendedGiftInfo: true,
});

conn.on(WebcastEvent.GIFT, (ev) => {
  try {
    const giftType = ev.gift?.type ?? ev.extendedGiftInfo?.type ?? 0;
    // gift type 1 = ส่งแบบ streak/combo -> รอจนจบ streak (repeatEnd) ค่อยส่งครั้งเดียว
    if (giftType === 1 && !ev.repeatEnd) return;
    emit({
      type: 'gift',
      user: ev.user?.nickname || ev.user?.uniqueId || 'ผู้ชม',
      unique_id: ev.user?.uniqueId || '',
      gift_id: Number(ev.giftId || ev.gift?.id || 0),
      gift_name: ev.gift?.name || ev.extendedGiftInfo?.name || '',
      diamonds: Number(ev.gift?.diamondCount ?? ev.extendedGiftInfo?.diamond_count ?? 0),
      repeat: Number(ev.repeatCount || 1),
      avatar: String(avatarOf(ev.user) || ''),
    });
  } catch (e) { console.error('[bridge] gift parse error', e); }
});

function giftImage(g) {
  return g?.image?.url_list?.[0] || g?.icon?.url_list?.[0] || g?.image?.url || g?.icon?.url || g?.image || '';
}

function avatarOf(u) {
  return u?.avatarUrl || u?.avatar_url || u?.avatarThumb?.url_list?.[0] || u?.avatar?.url_list?.[0]
    || u?.avatarThumb?.url || u?.avatar?.url || u?.profilePicUrl || u?.picture || '';
}

function normalizeGifts(list) {
  const arr = Array.isArray(list) ? list : (list?.gifts || list?.data?.gifts || []);
  const seen = new Set();
  const out = [];
  for (const g of arr) {
    const id = Number(g?.id || g?.gift_id || g?.giftId || 0);
    if (!id || seen.has(id)) continue;
    seen.add(id);
    out.push({
      id,
      name: String(g?.name || g?.gift_name || g?.giftName || ''),
      diamond_count: Number(g?.diamond_count ?? g?.diamondCount ?? g?.diamonds ?? 0),
      image: String(giftImage(g) || ''),
    });
  }
  return out;
}

async function emitGiftList(reason) {
  try {
    let list = conn.availableGifts;
    if (!list || (Array.isArray(list) && list.length === 0)) {
      list = await conn.fetchAvailableGifts();
    }
    const gifts = normalizeGifts(list);
    emit({
      type: 'gift_list',
      source: 'tiktok-live-connector fetchAvailableGifts',
      fetched_at: new Date().toISOString(),
      reason: reason || 'connect',
      gifts,
    });
  } catch (e) {
    emit({
      type: 'gift_list',
      source: 'tiktok-live-connector fetchAvailableGifts',
      fetched_at: new Date().toISOString(),
      gifts: [],
      error: String(e?.message || e),
    });
  }
}

// Settings window can write {"cmd":"refresh_gifts"} to stdin.
import('node:readline').then(({ createInterface }) => {
  const rl = createInterface({ input: process.stdin });
  rl.on('line', (line) => {
    const s = String(line || '').trim();
    if (!s) return;
    try {
      const msg = JSON.parse(s);
      if (msg?.cmd === 'refresh_gifts' || msg?.type === 'refresh_gifts') emitGiftList('refresh');
    } catch { /* ignore non-JSON */ }
  });
}).catch(() => {});

conn.on(ControlEvent.CONNECTED, (s) => {
  emit({ type: 'status', status: 'connected', room_id: s?.roomId ?? '' });
  emitGiftList('connect');
});
conn.on(ControlEvent.DISCONNECTED, () => { emit({ type: 'status', status: 'disconnected' }); process.exit(3); });
conn.on(WebcastEvent.STREAM_END, () => { emit({ type: 'status', status: 'stream_end' }); process.exit(4); });
conn.on(ControlEvent.ERROR, (e) => console.error('[bridge] error:', e?.message || e));

emit({ type: 'status', status: 'connecting', username });
conn.connect().catch((e) => {
  emit({ type: 'status', status: 'connect_failed', error: String(e?.message || e) });
  process.exit(1);
});

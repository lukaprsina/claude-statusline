// statusLine: `ctx 35.3k/1M 4% · 4:49`
//   ctx    context_window from the payload (filled right after /resume too)
//   clock  prompt_cache.expires_at; before a session's first response (a fresh
//          or just-resumed one) it falls back to the transcript's last assistant
//          message + 5 min, shown as ~m:ss
import { readFileSync, openSync, readSync, fstatSync, closeSync } from 'node:fs'

const TTL_MS = 300_000
const TAIL_BYTES = 256 * 1024

const fmtTokens = n => (n < 1000 ? String(n) : n < 1_000_000 ? `${(n / 1000).toFixed(1).replace(/\.0$/, '')}k` : `${(n / 1_000_000).toFixed(1).replace(/\.0$/, '')}M`)
const fmtClock = ms => {
  const s = Math.max(0, Math.ceil(ms / 1000))
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`
}

function lastAssistantAt(path) {
  const fd = openSync(path, 'r')
  try {
    const size = fstatSync(fd).size
    const len = Math.min(size, TAIL_BYTES)
    const buf = Buffer.alloc(len)
    readSync(fd, buf, 0, len, size - len)
    const lines = buf.toString('utf8').split('\n')
    for (let i = lines.length - 1; i >= 0; i--) {
      try {
        const o = JSON.parse(lines[i])
        if (o.type === 'assistant' && !o.isSidechain && o.timestamp) return Date.parse(o.timestamp)
      } catch {}
    }
  } finally {
    closeSync(fd)
  }
}

let p = {}
try { p = JSON.parse(readFileSync(0, 'utf8')) } catch {}

const cw = p.context_window ?? {}
const tokens = cw.total_input_tokens
const ctx = cw.used_percentage == null || !tokens ? 'ctx --' : `ctx ${fmtTokens(tokens)}/${fmtTokens(cw.context_window_size)} ${Math.round(cw.used_percentage)}%`

let clock = ''
const now = Date.now()
if (p.prompt_cache?.expires_at) {
  const left = p.prompt_cache.expires_at * 1000 - now
  clock = left > 0 ? fmtClock(left) : 'cache expired'
} else if (tokens && p.transcript_path) {
  try {
    const at = lastAssistantAt(p.transcript_path)
    if (at) clock = at + TTL_MS - now > 0 ? `~${fmtClock(at + TTL_MS - now)}` : 'cache expired'
  } catch {}
}

console.log(clock ? `${ctx} · ${clock}` : ctx)

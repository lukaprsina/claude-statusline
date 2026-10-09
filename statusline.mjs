// statusLine: `ctx 35.3k/1M 4% · 4:49`
//   ctx    context_window from the payload (filled right after /resume too)
//   clock  prompt_cache.expires_at; absent until a session's first response
import { readFileSync } from 'node:fs'

const fmtTokens = n => (n < 1000 ? String(n) : n < 1_000_000 ? `${(n / 1000).toFixed(1).replace(/\.0$/, '')}k` : `${(n / 1_000_000).toFixed(1).replace(/\.0$/, '')}M`)
const fmtClock = ms => {
  const s = Math.max(0, Math.ceil(ms / 1000))
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`
}

let p = {}
try { p = JSON.parse(readFileSync(0, 'utf8')) } catch {}

const cw = p.context_window ?? {}
const tokens = cw.total_input_tokens
const ctx = cw.used_percentage == null || !tokens ? 'ctx --' : `ctx ${fmtTokens(tokens)}/${fmtTokens(cw.context_window_size)} ${Math.round(cw.used_percentage)}%`

let clock = ''
if (p.prompt_cache?.expires_at) {
  const left = p.prompt_cache.expires_at * 1000 - Date.now()
  clock = left > 0 ? fmtClock(left) : 'cache expired'
}

console.log(clock ? `${ctx} · ${clock}` : ctx)

//! Claude Code statusLine: `ctx 35.3k/1M 4% · 4:49`
//!
//!   ctx    context_window from the stdin payload (filled right after /resume too)
//!   clock  prompt_cache.expires_at; before a session's first response it falls
//!          back to the transcript's last assistant message + 5 min, shown as ~m:ss

use serde_json::Value;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::time::{SystemTime, UNIX_EPOCH};

const TTL_MS: i64 = 300_000;
const TAIL_BYTES: u64 = 256 * 1024;

fn fmt_tokens(n: f64) -> String {
    let trim = |x: f64| format!("{x:.1}").trim_end_matches(".0").to_string();
    if n < 1000.0 {
        format!("{}", n as i64)
    } else if n < 1_000_000.0 {
        format!("{}k", trim(n / 1000.0))
    } else {
        format!("{}M", trim(n / 1_000_000.0))
    }
}

fn fmt_clock(ms: i64) -> String {
    let s = (ms.max(0) + 999) / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

/// "2026-10-06T14:29:01.719Z" to epoch milliseconds.
fn parse_iso(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    let num = |from: usize, to: usize| s.get(from..to)?.parse::<i64>().ok();
    if b.len() < 19 {
        return None;
    }
    let (y, m, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (hh, mm, ss) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    let millis = if b.get(19) == Some(&b'.') { num(20, 23).unwrap_or(0) } else { 0 };
    // days since 1970-01-01 (Howard Hinnant's civil-from-days, inverted)
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(((days * 24 + hh) * 60 + mm) * 60_000 + ss * 1000 + millis)
}

fn last_assistant_at(path: &str) -> Option<i64> {
    let mut f = File::open(path).ok()?;
    let size = f.metadata().ok()?.len();
    let len = size.min(TAIL_BYTES);
    f.seek(SeekFrom::Start(size - len)).ok()?;
    let mut buf = Vec::with_capacity(len as usize);
    f.read_to_end(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf);
    for line in text.lines().rev() {
        if !line.contains("\"type\":\"assistant\"") {
            continue;
        }
        let Ok(o) = serde_json::from_str::<Value>(line) else { continue };
        if o["type"] == "assistant" && o["isSidechain"] != true {
            if let Some(at) = o["timestamp"].as_str().and_then(parse_iso) {
                return Some(at);
            }
        }
    }
    None
}

fn main() {
    let mut raw = String::new();
    let _ = std::io::stdin().read_to_string(&mut raw);
    let p: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);

    let cw = &p["context_window"];
    let tokens = cw["total_input_tokens"].as_f64().unwrap_or(0.0);
    let ctx = match (cw["used_percentage"].as_f64(), tokens > 0.0) {
        (Some(pct), true) => format!(
            "ctx {}/{} {}%",
            fmt_tokens(tokens),
            fmt_tokens(cw["context_window_size"].as_f64().unwrap_or(0.0)),
            pct.round() as i64
        ),
        _ => "ctx --".to_string(),
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);
    let mut clock = String::new();
    if let Some(exp) = p["prompt_cache"]["expires_at"].as_i64().filter(|&e| e != 0) {
        let left = exp * 1000 - now;
        clock = if left > 0 { fmt_clock(left) } else { "cache expired".into() };
    } else if tokens > 0.0 {
        if let Some(at) = p["transcript_path"].as_str().and_then(last_assistant_at) {
            let left = at + TTL_MS - now;
            clock = if left > 0 { format!("~{}", fmt_clock(left)) } else { "cache expired".into() };
        }
    }

    if clock.is_empty() {
        println!("{ctx}");
    } else {
        println!("{ctx} · {clock}");
    }
}

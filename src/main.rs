//! Claude Code statusLine: `ctx 35.3k/1M 4% · 4:49`
//!
//!   ctx    context_window from the stdin payload (filled right after /resume too)
//!   clock  prompt_cache.expires_at; absent until a session's first response

use serde_json::Value;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

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
    }

    if clock.is_empty() {
        println!("{ctx}");
    } else {
        println!("{ctx} · {clock}");
    }
}

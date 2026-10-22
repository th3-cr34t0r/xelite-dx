use chrono::{TimeZone, Utc};
use dioxus::prelude::*;

// Cyber-data design palette (ui-designs/*.svg)
pub const BG: &str = "#0a0e17";
pub const PANEL: &str = "#0d1220";
pub const HAIR: &str = "#1a2340";
pub const ROW: &str = "#101828";
pub const ROWHAIR: &str = "#1f2b47";
pub const MINT: &str = "#64ffda";
pub const AMBER: &str = "#ffd700";
pub const TEXT: &str = "#ccd6f6";
pub const SUB: &str = "#a8b2d1";
pub const DIM: &str = "#4a5578";
pub const MONO: &str = "'Courier New',monospace";

/// Thin grid overlay matching the SVG design decoration (200px vertical / 150px horizontal lines)
pub fn grid_overlay() -> Element {
    rsx! {
        div {
            style: "position:fixed; top:0; left:0; right:0; bottom:0; pointer-events:none; z-index:0; background-image:repeating-linear-gradient(to right, rgba(26,35,64,0.45) 0px, rgba(26,35,64,0.45) 1px, transparent 1px, transparent 200px), repeating-linear-gradient(to bottom, rgba(26,35,64,0.45) 0px, rgba(26,35,64,0.45) 1px, transparent 1px, transparent 150px);",
        }
    }
}

/// Decorative 8px hex-cell grid (mint/amber/dark squares) as seen in the designs
pub fn hex_pattern(rows: usize, seed: usize, opacity: f32) -> Element {
    let cells: Vec<&'static str> = (0..rows * 8)
        .map(|i| match (i * 7 + seed * 13) % 5 {
            0 | 1 | 2 => MINT,
            3 => AMBER,
            _ => HAIR,
        })
        .collect();
    rsx! {
        div {
            style: format!("display:grid; grid-template-columns:repeat(8,8px); gap:4px; opacity:{opacity};"),
            for c in cells {
                div { style: format!("width:8px; height:8px; border-radius:2px; background-color:{c};") }
            }
        }
    }
}

/// Monogram initials for avatar circles: "ghost_protocol" -> GP, "cipher_ninja" -> CN
pub fn monogram(name: &str) -> String {
    let words: Vec<&str> = name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    if words.len() >= 2 {
        let a = words[0].chars().next().unwrap_or(' ').to_uppercase().to_string();
        let b = words[1].chars().next().unwrap_or(' ').to_uppercase().to_string();
        a + &b
    } else if let Some(w) = words.first() {
        w.chars().take(2).collect::<String>().to_uppercase()
    } else {
        String::new()
    }
}

/// Circular monogram avatar
pub fn avatar(name: &str, size: u32, active: bool) -> Element {
    let border = if active { MINT } else { ROWHAIR };
    rsx! {
        div {
            style: format!(
                "min-width:{size}px; width:{size}px; height:{size}px; border-radius:50%; background-color:{ROW}; border:1px solid {border}; color:{MINT}; font-size:{}px; display:flex; align-items:center; justify-content:center;",
                size * 40 / 100
            ),
            "{monogram(name)}"
        }
    }
}

/// Small uppercase tracked label
pub fn label(text: &'static str, extra: &str) -> Element {
    rsx! {
        div {
            style: format!("font-size:11px; color:{DIM}; letter-spacing:2px; font-family:{MONO}; {extra}"),
            "{text}"
        }
    }
}

/// HH:MM from a unix timestamp
pub fn fmt_clock(ts: i64) -> String {
    if ts <= 0 {
        return String::new();
    }
    Utc.timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.format("%H:%M").to_string())
        .unwrap_or_default()
}

/// "21 OCT 2026" from a unix timestamp
pub fn fmt_date(ts: i64) -> String {
    if ts <= 0 {
        return String::new();
    }
    Utc.timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.format("%d %b %Y").to_string().to_uppercase())
        .unwrap_or_default()
}

/// 0x7a3f…e91d style truncation
pub fn short_addr(addr: &str) -> String {
    if addr.len() > 24 {
        format!("{}…{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

/// 1,284,500 with thousands separators
pub fn fmt_num(n: i64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// 482900 -> "482.9K", 1284500 -> "1.3M"
pub fn fmt_k(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1000 {
        format!("{:.1}K", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

/// Copy text to the system clipboard (wl-copy on Wayland, xclip fallback)
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    use std::io::Write;

    let output = std::process::Command::new("which")
        .arg("wl-copy")
        .output()
        .map_err(|_| "wl-copy not found".to_string())?;

    if !output.status.success() {
        let output = std::process::Command::new("which")
            .arg("xclip")
            .output()
            .map_err(|_| "Neither wl-copy nor xclip found".to_string())?;

        if !output.status.success() {
            return Err("No clipboard tool available (need wl-copy or xclip)".to_string());
        }

        return std::process::Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
                child.wait()?;
                Ok(())
            })
            .map_err(|e| format!("xclip failed: {}", e));
    }

    std::process::Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
            child.wait()?;
            Ok(())
        })
        .map_err(|e| format!("wl-copy failed: {}", e))
}

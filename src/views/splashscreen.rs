use crate::database::db_fns::db_open_wallet;
use crate::views::theme::{grid_overlay, hex_pattern, BG, DIM, MINT, MONO, TEXT};
use dioxus::prelude::*;
use std::time::{Duration, Instant};

#[component]
pub fn SplashScreen() -> Element {
    use_future(move || async move {
        db_open_wallet().await;
    });

    // Animated progress bar (CSS keyframes don't load in desktop mode)
    let mut bar_width = use_signal(|| 60.0f32);
    use_future(move || async move {
        let start = Instant::now();
        loop {
            let t = start.elapsed().as_secs_f32();
            bar_width.set(60.0 + 55.0 * (t * std::f32::consts::PI).sin());
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    rsx! {
        div {
            style: format!("position:relative; display:flex; flex-direction:column; align-items:center; justify-content:center; min-height:100vh; background-color:{BG}; color:{TEXT}; font-family:{MONO}; overflow:hidden;"),

            {grid_overlay()}

            // Faint hex data-stream decoration (top-right, as in 01_splashscreen_desktop.svg)
            div {
                style: format!("position:absolute; top:16px; right:20px; font-size:8px; color:{MINT}; opacity:0.08; letter-spacing:1px; pointer-events:none; max-width:320px; word-break:break-all;"),
                "7a3f9d2c81b4e05f16c9a3d82f047be5c1a83f29d0e6b47c5219aef0d38b6c247015ea9d3c8f2b06a14e79d502c8b3fa16e409"
            }

            div { style: "position:relative; z-index:1; display:flex; flex-direction:column; align-items:center;",
                h1 {
                    style: format!("font-size:56px; font-weight:bold; margin:0 0 8px 0; color:{MINT}; letter-spacing:8px; text-shadow:0 0 24px rgba(100,255,218,0.35);"),
                    "XELITE"
                }
                div {
                    style: format!("font-size:14px; color:{MINT}; letter-spacing:10px; margin-bottom:16px; opacity:0.7;"),
                    "DX"
                }

                p {
                    style: format!("font-size:10px; color:{DIM}; letter-spacing:3px; margin:0 0 48px 0;"),
                    "v0.0.1 · SECURE ENCRYPTED MESSAGING"
                }

                // Animated progress bar
                div {
                    style: format!("width:120px; height:3px; background-color:#1a2340; border-radius:2px; overflow:hidden; margin-bottom:14px;"),
                    div {
                        style: format!("width:{bar_width}px; height:3px; background-color:{MINT}; box-shadow:0 0 8px rgba(100,255,218,0.6);"),
                    }
                }

                div {
                    style: format!("font-size:10px; color:{MINT}; letter-spacing:3px; margin-bottom:28px;"),
                    "CONNECTING..."
                }

                // Status indicators
                div { style: "display:flex; gap:24px;",
                    div { style: format!("display:flex; align-items:center; gap:8px; font-size:9px; color:{DIM}; letter-spacing:1px;"),
                        span { style: format!("color:{MINT};"), "●" }
                        "NODE SYNC"
                    }
                    div { style: format!("display:flex; align-items:center; gap:8px; font-size:9px; color:{DIM}; letter-spacing:1px;"),
                        span { style: format!("color:#ffd700;"), "●" }
                        "PEERS"
                    }
                }

                div { style: "margin-top:40px;", {hex_pattern(3, 1, 0.25)} }
            }
        }
    }
}

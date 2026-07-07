//! Scanner page — trending tokens table.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use serde_json::Value;
use crate::api;

#[component]
pub fn Scanner() -> impl IntoView {
    let (tokens, set_tokens) = signal::<Vec<Value>>(Vec::new());
    let (mode, set_mode) = signal("trending".to_string());

    let fetch = move |m: &str| {
        let m = m.to_string();
        spawn_local(async move {
            let data = match m.as_str() {
                "trenches" => api::fetch_trenches().await,
                _ => api::fetch_trending().await,
            };
            if let Ok(json) = data {
                if let Some(arr) = json.get("tokens").and_then(|t| t.as_array()) {
                    set_tokens.set(arr.clone());
                }
            }
        });
    };

    fetch("trending");

    let switch_mode = move |m: &str| {
        set_mode.set(m.to_string());
        fetch(m);
    };

    view! {
        <div class="space-y-4">
            <div class="flex items-center gap-2 mb-4">
                <h2 class="text-2xl font-bold text-white">"Scanner"</h2>
                <div class="flex gap-1 ml-4">
                    {["trending", "trenches"].iter().map(|m| {
                        let is_active = move || mode.get() == *m;
                        let m2 = m.to_string();
                        view! {
                            <button
                                class=move || {
                                    if is_active() { "px-3 py-1 rounded text-sm bg-blue-600 text-white".to_string() }
                                    else { "px-3 py-1 rounded text-sm bg-gray-800 text-gray-400 hover:bg-gray-700".to_string() }
                                }
                                on:click=move |_| switch_mode(&m2)
                            >
                                {*m}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            <div class="card overflow-x-auto">
                <table class="w-full">
                    <thead>
                        <tr class="border-b border-gray-800">
                            <th class="table-header">"Symbol"</th>
                            <th class="table-header">"Price"</th>
                            <th class="table-header">"M.Cap"</th>
                            <th class="table-header">"5m Vol"</th>
                            <th class="table-header">"1h Vol"</th>
                            <th class="table-header">"5m Chg"</th>
                            <th class="table-header">"1h Chg"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let items: Vec<Value> = tokens.with(|t| t.clone()).into_iter().take(30).collect();
                            items.into_iter().map(|t| {
                                let symbol = t.get("symbol").and_then(|s| s.as_str()).map(|s| s.to_string()).unwrap_or_else(|| "???".to_string());
                                let price = t.get("price_usd").and_then(|p| p.as_f64()).unwrap_or(0.0);
                                let mc = t.get("market_cap").and_then(|m| m.as_f64()).unwrap_or(0.0);
                                let v5m = t.get("volume_5m").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let v1h = t.get("volume_1h").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let c5m = t.get("change_5m").and_then(|c| c.as_f64()).unwrap_or(0.0);
                                let c1h = t.get("change_1h").and_then(|c| c.as_f64()).unwrap_or(0.0);
                                let address = t.get("address").and_then(|a| a.as_str()).map(|a| a.to_string()).unwrap_or_default();
                                let path_str = format!("/analyze/{}", address);
                                let price_str = format!("${:.8}", price);
                                let mc_str = if mc >= 1_000_000.0 { format!("${:.1}M", mc/1_000_000.0) } else if mc >= 1_000.0 { format!("${:.0}K", mc/1_000.0) } else { format!("${:.0}", mc) };
                                let v5m_str = if v5m >= 1_000_000.0 { format!("${:.1}M", v5m/1_000_000.0) } else if v5m > 0.0 { format!("${:.0}K", v5m/1_000.0) } else { "-".to_string() };
                                let c5m_color = if c5m > 0.0 { "text-green-400" } else { "text-red-400" };
                                let c1h_color = if c1h > 0.0 { "text-green-400" } else { "text-red-400" };

                                view! {
                                    <tr class="border-b border-gray-800 hover:bg-gray-800">
                                        <td class="table-cell font-medium">
                                            <A href=path_str attr:class="text-blue-400 hover:underline">{symbol}</A>
                                        </td>
                                        <td class="table-cell text-gray-400">{price_str}</td>
                                        <td class="table-cell">{mc_str}</td>
                                        <td class="table-cell text-gray-400">{v5m_str}</td>
                                        <td class="table-cell text-gray-400">{v1h}</td>
                                        <td class=format!("table-cell {}", c5m_color)>{format!("{:+.2}%", c5m)}</td>
                                        <td class=format!("table-cell {}", c1h_color)>{format!("{:+.2}%", c1h)}</td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()
                        }}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

//! Analyzer page — token detail with alpha report.

use leptos::*;
use leptos_router::*;
use serde_json::Value;
use crate::api;

#[component]
pub fn Analyzer() -> impl IntoView {
    let params = use_params_map();
    let address = move || params.with(|p| p.get("address").cloned().unwrap_or_default());
    let (data, set_data) = create_signal::<Option<Value>>(None);

    spawn_local(async move {
        let addr = address();
        if !addr.is_empty() {
            if let Ok(res) = api::analyze_token(&addr).await {
                set_data.set(Some(res));
            }
        }
    });

    view! {
        <div class="space-y-6">
            <h2 class="text-2xl font-bold text-white">"Token Analysis"</h2>

            {move || data.get().map(|d| {
                let detail = d.get("detail");
                let report = d.get("report");

                let symbol = detail.and_then(|d| d.get("symbol").and_then(|s| s.as_str())).unwrap_or("Unknown");
                let name = detail.and_then(|d| d.get("name").and_then(|n| n.as_str())).unwrap_or("");
                let price = detail.and_then(|d| d.get("price_usd").and_then(|p| p.as_f64())).unwrap_or(0.0);
                let mc = detail.and_then(|d| d.get("market_cap").and_then(|m| m.as_f64())).unwrap_or(0.0);
                let alpha_score = report.and_then(|r| r.get("alpha_score").and_then(|a| a.as_f64())).unwrap_or(0.0);
                let rug_severity = report.and_then(|r| r.get("rug_report").and_then(|rg| rg.get("severity").and_then(|s| s.as_str()))).unwrap_or("Unknown");
                let verdict = report.and_then(|r| r.get("rug_report").and_then(|rg| rg.get("verdict").and_then(|v| v.as_str()))).unwrap_or("");

                let score_color = if alpha_score >= 70.0 { "text-green-400" } else if alpha_score >= 40.0 { "text-yellow-400" } else { "text-red-400" };

                view! {
                    <div class="grid grid-cols-2 gap-6">
                        <div class="card">
                            <h3 class="text-lg font-semibold text-white mb-2">{symbol} " - " {name}</h3>
                            <p class="text-gray-400">"Price: " <span class="text-white">"$" {format!("{:.8}", price)}</span></p>
                            <p class="text-gray-400">"Market Cap: " <span class="text-white">"$" {format!("{:.0}", mc)}</span></p>
                        </div>

                        <div class="card">
                            <h3 class="text-lg font-semibold text-white mb-2">"Alpha Score"</h3>
                            <p class=format!("text-4xl font-bold {}", score_color)>{format!("{:.0}", alpha_score)}</p>
                        </div>
                    </div>

                    <div class="card">
                        <h3 class="text-lg font-semibold text-white mb-2">"Rug Report"</h3>
                        <div class="flex items-center gap-2">
                            <span class=format!("badge {}", if rug_severity == "Low" { "badge-green" } else if rug_severity == "Medium" { "badge-yellow" } else { "badge-red" })>
                                {rug_severity}
                            </span>
                            <span class="text-gray-400 text-sm">{verdict}</span>
                        </div>
                    </div>
                }
            })}

            {move || if data.get().is_none() {
                view! { <p class="text-gray-500">"Loading token analysis..."</p> }
            } else {
                view! { <></> }
            }}
        </div>
    }
}

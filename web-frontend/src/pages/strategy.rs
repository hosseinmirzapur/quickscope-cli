//! Strategy page — alpha filter configuration.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::Value;
use crate::api;

#[component]
pub fn Strategy() -> impl IntoView {
    let (config, set_config) = signal::<Option<Value>>(None);

    spawn_local(async move {
        if let Ok(data) = api::fetch_strategy().await {
            set_config.set(data.get("strategy").cloned());
        }
    });

    view! {
        <div class="space-y-6">
            <h2 class="text-2xl font-bold text-white">"Alpha Strategy"</h2>

            <div class="card max-w-lg">
                <h3 class="text-lg font-semibold text-white mb-3">"Filter Weights"</h3>
                {move || config.get().map(|c| {
                    let fields = ["w_momentum", "w_safety", "w_holder", "w_liquidity", "w_dev", "w_social"];
                    fields.iter().map(|f| {
                        let val = c.get(*f).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let label = f.replace("w_", "");
                        let pct = (val * 100.0) as u32;
                        view! {
                            <div class="flex items-center justify-between py-2 border-b border-gray-800 last:border-0">
                                <span class="text-gray-400 text-sm capitalize">{label.clone()}</span>
                                <div class="flex items-center gap-2">
                                    <div class="w-32 h-2 bg-gray-700 rounded-full overflow-hidden">
                                        <div class="h-full bg-blue-500 rounded-full" style=format!("width: {}%", pct)></div>
                                    </div>
                                    <span class="text-white text-sm w-8 text-right">{pct}"%"</span>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                })}
            </div>

            <div class="card max-w-lg">
                <h3 class="text-lg font-semibold text-white mb-3">"Hard Filters"</h3>
                {move || config.get().map(|c| {
                    let hf_fields = ["hf_rug_ratio_max", "hf_dev_hold_max", "hf_liquidity_min_usd"];
                    hf_fields.iter().map(|f| {
                        let val = c.get(*f).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let label = f.replace("hf_", "").replace("_", " ");
                        view! {
                            <div class="flex items-center justify-between py-2 border-b border-gray-800 last:border-0">
                                <span class="text-gray-400 text-sm capitalize">{label}</span>
                                <span class="text-white text-sm">{val}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                })}
            </div>
        </div>
    }
}

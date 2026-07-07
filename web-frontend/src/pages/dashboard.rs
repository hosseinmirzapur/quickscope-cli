//! Dashboard page — overview of portfolio, signals, and activity.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::Value;
use crate::api;

#[component]
pub fn Dashboard() -> impl IntoView {
    let (positions, set_positions) = signal::<Option<Value>>(None);
    let (settings, set_settings) = signal::<Option<Value>>(None);
    let (trending, set_trending) = signal::<Option<Value>>(None);

    spawn_local(async move {
        if let Ok(data) = api::fetch_positions().await {
            set_positions.set(Some(data));
        }
        if let Ok(data) = api::fetch_settings().await {
            set_settings.set(Some(data));
        }
        if let Ok(data) = api::fetch_trending().await {
            set_trending.set(Some(data));
        }
    });

    view! {
        <div class="space-y-6">
            <h2 class="text-2xl font-bold text-white">"Dashboard"</h2>

            <div class="grid grid-cols-3 gap-4">
                <div class="card">
                    <p class="text-sm text-gray-400">"Balance"</p>
                    <p class="text-2xl font-bold text-green-400">
                        {move || settings.get()
                            .and_then(|s| s.get("balance").and_then(|b| b.as_f64()))
                            .map(|b| format!("{:.2} SOL", b))
                            .unwrap_or_else(|| "Loading...".to_string())
                        }
                    </p>
                </div>
                <div class="card">
                    <p class="text-sm text-gray-400">"Open Positions"</p>
                    <p class="text-2xl font-bold text-blue-400">
                        {move || positions.with(|p| {
                            p.as_ref()
                                .and_then(|p| p.get("positions"))
                                .and_then(|arr| arr.as_array())
                                .map(|arr| arr.len().to_string())
                                .unwrap_or_else(|| "0".to_string())
                        })}
                    </p>
                </div>
                <div class="card">
                    <p class="text-sm text-gray-400">"Trending Tokens"</p>
                    <p class="text-2xl font-bold text-yellow-400">
                        {move || trending.with(|t| {
                            t.as_ref()
                                .and_then(|t| t.get("tokens"))
                                .and_then(|arr| arr.as_array())
                                .map(|arr| arr.len().to_string())
                                .unwrap_or_else(|| "0".to_string())
                        })}
                    </p>
                </div>
            </div>

            <div class="card">
                <h3 class="text-lg font-semibold text-white mb-3">"Recent Signals"</h3>
                <p class="text-gray-500 text-sm">"Connect to WebSocket for real-time signals."</p>
            </div>
        </div>
    }
}

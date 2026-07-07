//! Settings page — API keys, theme, log level.

use leptos::*;
use serde_json::Value;
use crate::api;

#[component]
pub fn Settings() -> impl IntoView {
    let (settings, set_settings) = create_signal::<Option<Value>>(None);

    spawn_local(async move {
        if let Ok(data) = api::fetch_settings().await {
            set_settings.set(Some(data));
        }
    });

    view! {
        <div class="space-y-6">
            <h2 class="text-2xl font-bold text-white">"Settings"</h2>

            <div class="card max-w-lg">
                <h3 class="text-lg font-semibold text-white mb-3">"API Keys"</h3>
                {move || settings.get().and_then(|s| s.get("api_keys").cloned()).map(|keys| {
                    let names = ["alph_dex", "openai", "anthropic", "ollama"];
                    names.iter().map(|k| {
                        let status = keys.get(*k).and_then(|v| v.as_str()).unwrap_or("missing");
                        let dot_color = if status == "configured" { "bg-green-400" } else { "bg-red-400" };
                        view! {
                            <div class="flex items-center justify-between py-2 border-b border-gray-800 last:border-0">
                                <span class="text-gray-400 text-sm capitalize">{k.replace("_", " ")}</span>
                                <div class="flex items-center gap-2">
                                    <span class=format!("w-2 h-2 rounded-full {}", dot_color)></span>
                                    <span class="text-sm text-gray-500">{status}</span>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                })}
            </div>

            <div class="card max-w-lg">
                <h3 class="text-lg font-semibold text-white mb-3">"General"</h3>
                <div class="space-y-3">
                    <div>
                        <label class="text-sm text-gray-400 block mb-1">"Theme"</label>
                        <p class="text-white">"Dark (default)"</p>
                    </div>
                    <div>
                        <label class="text-sm text-gray-400 block mb-1">"Log Level"</label>
                        <p class="text-white">
                            {move || settings.get().and_then(|s| s.get("log_level").and_then(|l| l.as_str())).unwrap_or("info")}
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}

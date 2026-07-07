//! Trade page — paper trading interface.

use leptos::*;
use serde_json::Value;
use crate::api;

#[component]
pub fn Trade() -> impl IntoView {
    let (positions, set_positions) = create_signal::<Vec<Value>>(Vec::new());
    let (token_addr, set_token_addr) = create_signal("".to_string());
    let (amount, set_amount) = create_signal(0.1f64);
    let (mode, set_mode) = create_signal("ALPHA".to_string());
    let (result_msg, set_result_msg) = create_signal("".to_string());

    let fetch_positions = move || {
        spawn_local(async move {
            if let Ok(data) = api::fetch_positions().await {
                if let Some(arr) = data.get("positions").and_then(|p| p.as_array()) {
                    set_positions.set(arr.clone());
                }
            }
        });
    };
    fetch_positions();

    let do_buy = move |_| {
        let addr = token_addr.get();
        let amt = amount.get();
        let m = mode.get();
        set_result_msg.set("Executing...".to_string());
        spawn_local(async move {
            match api::buy_paper(&addr, amt, &m, None, None).await {
                Ok(res) => {
                    let msg = res.get("tokens_received")
                        .and_then(|t| t.as_f64())
                        .map(|t| format!("Bought! Received {:.2} tokens", t))
                        .unwrap_or_else(|| "Buy executed".to_string());
                    set_result_msg.set(msg);
                    fetch_positions();
                }
                Err(e) => set_result_msg.set(format!("Error: {}", e)),
            }
        });
    };

    view! {
        <div class="space-y-6">
            <h2 class="text-2xl font-bold text-white">"Paper Trade"</h2>

            <div class="card max-w-lg">
                <h3 class="text-lg font-semibold text-white mb-3">"New Trade"</h3>
                <div class="space-y-3">
                    <div>
                        <label class="text-sm text-gray-400 block mb-1">"Token Address"</label>
                        <input class="input" type="text" placeholder="Enter token address"
                            on:input=move |e| set_token_addr.set(event_target_value(&e)) />
                    </div>
                    <div>
                        <label class="text-sm text-gray-400 block mb-1">"Amount (SOL)"</label>
                        <input class="input" type="number" step="0.1" min="0.1" value="0.1"
                            on:input=move |e| set_amount.set(event_target_value(&e).parse().unwrap_or(0.1)) />
                    </div>
                    <div>
                        <label class="text-sm text-gray-400 block mb-1">"Mode"</label>
                        <select class="input" on:change=move |e| set_mode.set(event_target_value(&e))>
                            <option value="EXPLODE">"EXPLODE"</option>
                            <option value="ALPHA" selected>"ALPHA"</option>
                            <option value="SCALP">"SCALP"</option>
                            <option value="FALLBACK">"FALLBACK"</option>
                        </select>
                    </div>
                    <button class="btn btn-primary w-full" on:click=do_buy>"Buy"</button>
                    {move || if !result_msg.get().is_empty() {
                        view! { <p class="text-sm text-green-400">{result_msg.get()}</p> }
                    } else {
                        view! { <></> }
                    }}
                </div>
            </div>

            <div class="card">
                <h3 class="text-lg font-semibold text-white mb-3">"Open Positions"</h3>
                <table class="w-full">
                    <thead>
                        <tr class="border-b border-gray-800">
                            <th class="table-header">"Symbol"</th>
                            <th class="table-header">"Entry"</th>
                            <th class="table-header">"Amount"</th>
                            <th class="table-header">"Mode"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || positions.get().iter().map(|p| {
                            let symbol = p.get("token_symbol").and_then(|s| s.as_str()).unwrap_or("?");
                            let entry = p.get("entry_price").and_then(|e| e.as_f64()).unwrap_or(0.0);
                            let amt = p.get("amount_sol").and_then(|a| a.as_f64()).unwrap_or(0.0);
                            let mode = p.get("mode").and_then(|m| m.as_str()).unwrap_or("?");
                            view! {
                                <tr class="border-b border-gray-800">
                                    <td class="table-cell font-medium">{symbol}</td>
                                    <td class="table-cell text-gray-400">{format!("${:.8}", entry)}</td>
                                    <td class="table-cell">{format!("{:.2} SOL", amt)}</td>
                                    <td class="table-cell">
                                        <span class="badge badge-green">{mode}</span>
                                    </td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

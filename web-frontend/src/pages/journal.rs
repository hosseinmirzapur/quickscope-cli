//! Journal page — trade history.

use leptos::*;
use serde_json::Value;
use crate::api;

#[component]
pub fn Journal() -> impl IntoView {
    let (entries, set_entries) = create_signal::<Vec<Value>>(Vec::new());

    spawn_local(async move {
        if let Ok(data) = api::fetch_journal().await {
            if let Some(arr) = data.get("journal").and_then(|j| j.as_array()) {
                set_entries.set(arr.clone());
            }
        }
    });

    view! {
        <div class="space-y-4">
            <h2 class="text-2xl font-bold text-white">"Trade Journal"</h2>
            <div class="card overflow-x-auto">
                <table class="w-full">
                    <thead>
                        <tr class="border-b border-gray-800">
                            <th class="table-header">"Symbol"</th>
                            <th class="table-header">"Entry"</th>
                            <th class="table-header">"Exit"</th>
                            <th class="table-header">"PnL"</th>
                            <th class="table-header">"Mode"</th>
                            <th class="table-header">"Status"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || entries.get().iter().map(|e| {
                            let symbol = e.get("token_symbol").and_then(|s| s.as_str()).unwrap_or("?");
                            let entry = e.get("entry_price").and_then(|ep| ep.as_f64()).unwrap_or(0.0);
                            let exit = e.get("exit_price").and_then(|ep| ep.as_f64());
                            let pnl = e.get("pnl_sol").and_then(|p| p.as_f64());
                            let mode = e.get("mode").and_then(|m| m.as_str()).unwrap_or("?");
                            let status = e.get("status").and_then(|s| s.as_str()).unwrap_or("?");
                            let pnl_str = pnl.map(|p| if p > 0.0 { format!("+{:.4}", p) } else { format!("{:.4}", p) }).unwrap_or_else(|| "-".to_string());
                            let pnl_color = pnl.map(|p| if p > 0.0 { "text-green-400" } else { "text-red-400" }).unwrap_or("text-gray-400");
                            let status_badge = if status == "open" { "badge-green" } else { "badge-yellow" };

                            view! {
                                <tr class="border-b border-gray-800">
                                    <td class="table-cell font-medium">{symbol}</td>
                                    <td class="table-cell text-gray-400">{format!("${:.8}", entry)}</td>
                                    <td class="table-cell text-gray-400">{exit.map(|e| format!("${:.8}", e)).unwrap_or_else(|| "-".to_string())}</td>
                                    <td class=format!("table-cell {}", pnl_color)>{pnl_str}</td>
                                    <td class="table-cell">{mode}</td>
                                    <td class="table-cell"><span class=format!("badge {}", status_badge)>{status}</span></td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

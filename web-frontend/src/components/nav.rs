//! Sidebar navigation component.

use leptos::*;
use leptos_router::*;

#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();

    let active_tab = move |path: &str| {
        if location.pathname.get() == path {
            "active"
        } else {
            ""
        }
    };

    let tabs = vec![
        ("/", "📊", "Dashboard"),
        ("/scanner", "🔍", "Scanner"),
        ("/trade", "💰", "Trade"),
        ("/journal", "📓", "Journal"),
        ("/strategy", "⚙️", "Strategy"),
        ("/settings", "🔧", "Settings"),
    ];

    view! {
        <nav class="w-48 bg-gray-900 border-r border-gray-800 flex flex-col py-4">
            <div class="px-4 mb-6">
                <h1 class="text-lg font-bold text-blue-400">"QuickScope"</h1>
                <p class="text-xs text-gray-500">"Alpha Scanner"</p>
            </div>
            {tabs.into_iter().map(|(path, icon, label)| {
                let class = format!("sidebar-tab {}", active_tab(path));
                view! {
                    <a href=path class=class>
                        <span>{icon}</span>
                        <span>{label}</span>
                    </a>
                }
            }).collect::<Vec<_>>()}
        </nav>
    }
}

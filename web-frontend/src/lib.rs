//! QuickScope Web Frontend — App root with routing.

pub mod api;
pub mod components;
pub mod pages;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use components::nav::Sidebar;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <div class="flex h-screen">
                <Sidebar />
                <main class="flex-1 overflow-auto p-6">
                    <Routes>
                        <Route path="/" view=|| view! { <pages::dashboard::Dashboard /> } />
                        <Route path="/scanner" view=|| view! { <pages::scanner::Scanner /> } />
                        <Route path="/analyze/:address" view=|| view! { <pages::analyzer::Analyzer /> } />
                        <Route path="/trade" view=|| view! { <pages::trade::Trade /> } />
                        <Route path="/journal" view=|| view! { <pages::journal::Journal /> } />
                        <Route path="/strategy" view=|| view! { <pages::strategy::Strategy /> } />
                        <Route path="/settings" view=|| view! { <pages::settings::Settings /> } />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

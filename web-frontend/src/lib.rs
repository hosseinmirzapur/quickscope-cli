//! QuickScope Web Frontend — App root with routing.

pub mod api;
pub mod components;
pub mod pages;

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use components::nav::Sidebar;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="flex h-screen">
                <Sidebar />
                <main class="flex-1 overflow-auto p-6">
                    <Routes fallback=|| "Page not found">
                        <Route path=path!("/") view=DashboardPage />
                        <Route path=path!("/scanner") view=ScannerPage />
                        <Route path=path!("/analyze/:address") view=AnalyzerPage />
                        <Route path=path!("/trade") view=TradePage />
                        <Route path=path!("/journal") view=JournalPage />
                        <Route path=path!("/strategy") view=StrategyPage />
                        <Route path=path!("/settings") view=SettingsPage />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

// Shim components to avoid the closure-vs-component confusion in leptos_router 0.7
#[component]
fn DashboardPage() -> impl IntoView { pages::dashboard::Dashboard() }
#[component]
fn ScannerPage() -> impl IntoView { pages::scanner::Scanner() }
#[component]
fn AnalyzerPage() -> impl IntoView { pages::analyzer::Analyzer() }
#[component]
fn TradePage() -> impl IntoView { pages::trade::Trade() }
#[component]
fn JournalPage() -> impl IntoView { pages::journal::Journal() }
#[component]
fn StrategyPage() -> impl IntoView { pages::strategy::Strategy() }
#[component]
fn SettingsPage() -> impl IntoView { pages::settings::Settings() }

//! Root application component with router.

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::path;

use crate::pages::converter::ConverterPage;
use crate::pages::coordinators::CoordinatorsPage;

/// Root application component.
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="navbar">
                <h1>"Automapper"</h1>
                <nav>
                    <A href="/">"Converter"</A>
                    <A href="/coordinators">"Coordinators"</A>
                    <a href="/swagger-ui/" target="_blank" rel="noopener">"API Docs"</a>
                </nav>
            </div>

            <main>
                <Routes fallback=|| view! { <p>"Page not found."</p> }>
                    <Route path=path!("/") view=ConverterPage />
                    <Route path=path!("/coordinators") view=CoordinatorsPage />
                </Routes>
            </main>
        </Router>
    }
}

use leptos::prelude::*;

#[component]
pub fn Users() -> impl IntoView {
    view! {
        <p>"Users"</p>
    }
}

#[component]
pub fn UserDetail() -> impl IntoView {
    view! {
        <p>"User Detail"</p>
    }
}

#[component]
pub fn NoUser() -> impl IntoView {
    view! {
        <p></p>
    }
}

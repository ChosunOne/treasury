use leptos::prelude::*;

#[component]
pub fn Transactions() -> impl IntoView {
    view! {
        <p>"Transactions"</p>
    }
}

#[component]
pub fn TransactionDetail() -> impl IntoView {
    view! {
        <p>"Transaction Detail"</p>
    }
}

#[component]
pub fn NoTransaction() -> impl IntoView {
    view! {
        <p></p>
    }
}

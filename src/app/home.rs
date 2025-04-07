use leptos::prelude::*;

use crate::{
    api::account_api::{AccountApiGetList, get_list},
    schema::{Pagination, account::GetListRequest},
};

#[component]
pub fn Home() -> impl IntoView {
    let accounts_action = ServerAction::<AccountApiGetList>::new();

    let accounts = Resource::new(
        move || accounts_action.version().get(),
        |_| {
            get_list(
                GetListRequest {
                    name: None,
                    institution_id: None,
                },
                Pagination::default(),
            )
        },
    );

    view! {
        <h1>"Accounts"</h1>
        <Suspense fallback=move || view! { <p>"Loading"</p> }>
            {move || {
                accounts.get()
                    .and_then(|a| a.ok())
                    .map(|a| a.accounts.iter()
                        .map(|_| view! {
                            <p>"An account!"</p>
                        }).collect_view()
                    )
                }
            }
        </Suspense>
    }
}

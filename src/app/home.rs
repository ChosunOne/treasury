use leptos::prelude::*;

use crate::{
    api::account_api::get_list,
    app::AuthToken,
    schema::{Pagination, account::GetListRequest},
};

#[component]
pub fn Home() -> impl IntoView {
    let auth_token = expect_context::<AuthToken>().0;

    let accounts = Resource::new(
        move || auth_token.get(),
        |auth_signal| async move {
            if auth_signal.is_none() {
                return vec![];
            }
            get_list(
                GetListRequest {
                    name: None,
                    institution_id: None,
                },
                Pagination::default(),
            )
            .await
            .expect("Failed to get accounts")
            .accounts
        },
    );

    view! {
        <Show when=move || auth_token.get().is_some() fallback=|| view! {<p>"Log in to access Treasury."</p>}>
            <div class="container">
                <div class="flex justify-center">
                    <Suspense fallback=move || view! { <p>"Loading"</p> }>
                        <table class="bg-indigo-50 table-auto">
                            <thead>
                                <tr>
                                    <th class="px-2 border border-gray-400">"Institution"</th>
                                    <th class="px-2 border border-gray-400">"Account Name"</th>
                                    <th class="px-2 border border-gray-400">"Account Value"</th>
                                    <th class="px-2 border border-gray-400">"% Change (1d)"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                        accounts.get().unwrap_or(vec![]).iter().enumerate()
                                            .map(|(i, a)| view! {
                                                <tr class={
                                                    if i % 2 == 0 {
                                                        "bg-indigo-50 border border-gray-300"
                                                    } else {
                                                        "bg-indigo-100 border border-gray-300"
                                                    }
                                                }>
                                                    <td class="px-2 text-center border border-gray-300">"Example Institution"</td>
                                                    <td class="px-2 text-center border border-gray-300">{a.name.clone()}</td>
                                                    <td class="px-4 text-right border border-gray-300">"1234.56"</td>
                                                    <td class="px-10 text-right border border-gray-300">"12.7"</td>
                                                </tr>
                                            }).collect_view()
                                    }
                                }
                            </tbody>
                        </table>
                    </Suspense>
                </div>
            </div>
        </Show>
    }
}

use std::collections::HashMap;

use leptos::prelude::*;

use crate::{
    api::{
        account_api::get_list as account_get_list,
        institution_api::get_list as institution_get_list,
    },
    app::AuthToken,
    schema::{
        Pagination, account::GetListRequest as AccountGetListRequest,
        institution::GetListRequest as InstitutionGetListRequest,
    },
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
            account_get_list(
                AccountGetListRequest {
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

    let institutions_map = Resource::new(
        move || auth_token.get(),
        |auth_signal| async move {
            if auth_signal.is_none() {
                return HashMap::new();
            }
            institution_get_list(
                InstitutionGetListRequest { name: None },
                Pagination::default(),
            )
            .await
            .expect("Failed to get institutions")
            .institutions
            .into_iter()
            .map(|i| (i.id, i))
            .collect()
        },
    );

    view! {
        <Show when=move || auth_token.get().is_some() fallback=|| view! {<p class="text-ctp-text">"Log in to access Treasury."</p>}>
            <div class="container">
                <div class="flex justify-center">
                    <Suspense fallback=move || view! { <p>"Loading"</p> }>
                        <table class="bg-ctp-base table-auto">
                            <thead>
                                <tr>
                                    <th class="text-ctp-yellow px-2 border border-ctp-surface2">"Institution"</th>
                                    <th class="text-ctp-blue px-2 border border-ctp-surface2">"Account Name"</th>
                                    <th class="text-ctp-green px-2 border border-ctp-surface2">"Account Value"</th>
                                    <th class="text-ctp-mauve px-2 border border-ctp-surface2">"% Change (1d)"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                        let institutions = institutions_map.get().unwrap_or(HashMap::new());
                                        accounts.get().unwrap_or(vec![]).iter().enumerate()
                                            .map(|(i, a)| view! {
                                                <tr class={
                                                    if i % 2 == 0 {
                                                        "bg-ctp-surface0 border border-ctp-surface2"
                                                    } else {
                                                        "bg-ctp-surface1 border border-ctp-surface2"
                                                    }
                                                }>
                                                    <td class="text-ctp-text px-2 text-center border border-ctp-surface2">{institutions.get(&a.institution_id).map(|i| i.name.clone()).unwrap_or("".to_owned())}</td>
                                                    <td class="text-ctp-text px-2 text-center border border-ctp-surface2">{a.name.clone()}</td>
                                                    <td class="text-ctp-text px-4 text-right border border-ctp-surface2">"1234.56"</td>
                                                    <td class="text-ctp-text px-10 text-right border border-ctp-surface2">"12.7"</td>
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

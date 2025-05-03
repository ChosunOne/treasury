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
pub fn Accounts() -> impl IntoView {
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
        <Suspense fallback=move || view! {<p>"Loading..."</p>}>
            <div class="container">
                {move || {
                    let Some(a_list) = accounts.get() else {return Vec::new();};
                    let Some(institutions) = institutions_map.get() else {return Vec::new();};
                    a_list.into_iter().filter(|a| {
                        institutions.contains_key(&a.institution_id)
                    }).map(|a| {
                        let institution_name = institutions.get(&a.institution_id).unwrap().name.clone();
                        view! {
                            <a href=format!("accounts/{}", a.id) class="block max-w-sm p-6 bg-white border border-gray-200 rounded-lg shadow-sm hover:bg-gray-100 dark:bg-gray-800 dark:border-gray-600 dark:hover:bg-gray-700">
                                <h5 class="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">{institution_name}</h5>
                                <p class="font-normal text-gray-700 dark:text-gray-400">{a.name}</p>
                            </a>
                        }
                    }).collect_view()
                }}
            </div>
        </Suspense>
    }
}

#[component]
pub fn AccountDetail() -> impl IntoView {
    view! {
        <p>"Account Detail"</p>
    }
}

#[component]
pub fn NoAccount() -> impl IntoView {
    view! {
        <p></p>
    }
}

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
        <p>"Accounts"</p>
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

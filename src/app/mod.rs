use leptos::prelude::*;
use leptos_meta::{MetaTags, Title, provide_meta_context};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    path,
};

use crate::app::{
    accounts::{AccountDetail, Accounts, NoAccount},
    assets::{AssetDetail, Assets, NoAsset},
    auth::{HandleAuth, Login, Logout, SsoRefresh},
    home::Home,
    institutions::{InstitutionDetail, Institutions, NoInstitution},
    transactions::{NoTransaction, TransactionDetail, Transactions},
    users::{NoUser, UserDetail, Users},
};

pub mod accounts;
pub mod assets;
pub mod auth;
pub mod home;
pub mod institutions;
pub mod transactions;
pub mod users;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <link rel="stylesheet" id="leptos" href="/pkg/treasury.css"/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[derive(Clone, Debug)]
pub struct AuthToken(pub RwSignal<Option<String>>);
#[derive(Clone, Debug)]
pub struct ExpiresIn(pub RwSignal<i64>);

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let rw_auth_token = RwSignal::<Option<String>, _>::new(None);
    let rw_expires_in = RwSignal::<i64, _>::new(0);

    provide_context(AuthToken(rw_auth_token));
    provide_context(ExpiresIn(rw_expires_in));

    let refresh_token = ServerAction::<SsoRefresh>::new();

    Effect::new(move |handle: Option<Option<TimeoutHandle>>| {
        if let Some(prev_handle) = handle.flatten() {
            prev_handle.clear();
        }

        let expires_in = rw_expires_in.get();
        if expires_in != 0 {
            let handle = set_timeout_with_handle(
                move || {
                    refresh_token.dispatch(SsoRefresh {});
                },
                std::time::Duration::from_secs(
                    expires_in.checked_sub(30).unwrap_or_default() as u64
                ),
            )
            .unwrap();
            Some(handle)
        } else {
            // See if we can get a token from an extant refresh token
            let handle = set_timeout_with_handle(
                move || {
                    refresh_token.dispatch(SsoRefresh {});
                },
                std::time::Duration::from_secs(0),
            )
            .unwrap();
            Some(handle)
        }
    });

    Effect::new(move |_| {
        let value = refresh_token.value();
        if let Some(Ok((ref auth_token, expires_in))) = *value.get() {
            rw_expires_in.set(expires_in);
            rw_auth_token.set(Some(auth_token.into()));
        }
    });

    view! {
        <Title text="Treasury"/>
        <Router>
            <nav class="mt-1 mb-1 ml-1 flex flex-row rounded-lg">
                <Show when=move || rw_auth_token.get().is_some() fallback=|| view! {
                    <div class="flex-auto"></div>
                    <Login/>
                }>
                    <button class="rounded border border-gray-300 bg-amber-50 px-4 py-2 font-medium transition hover:bg-amber-200 cursor-pointer">"Home"</button>
                    <button class="rounded border border-gray-300 bg-amber-50 px-4 py-2 font-medium transition hover:bg-amber-200 cursor-pointer">"Accounts"</button>
                    <button class="rounded border border-gray-300 bg-amber-50 px-4 py-2 font-medium transition hover:bg-amber-200 cursor-pointer">"Transactions"</button>
                    <div class="flex-auto"></div>
                    <button class="rounded border border-gray-300 bg-amber-50 px-4 py-2 font-medium transition hover:bg-amber-200 cursor-pointer">"Profile Options"</button>
                    <Logout/>
                </Show>
            </nav>


            <main>
                <Routes fallback=|| "This page could not be found.">
                    <Route path=path!("/oauth2-redirect") view=HandleAuth/>
                    <Route path=path!("/home") view=Home/>
                    <ParentRoute path=path!("/home/accounts") view=Accounts>
                        <Route path=path!(":id") view=AccountDetail/>
                        <Route path=path!("") view=NoAccount/>
                    </ParentRoute>
                    <ParentRoute path=path!("/home/users") view=Users>
                        <Route path=path!(":id") view=UserDetail/>
                        <Route path=path!("") view=NoUser/>
                    </ParentRoute>
                    <ParentRoute path=path!("/home/assets") view=Assets>
                        <Route path=path!(":id") view=AssetDetail/>
                        <Route path=path!("") view=NoAsset/>
                    </ParentRoute>
                    <ParentRoute path=path!("/home/institutions") view=Institutions>
                        <Route path=path!(":id") view=InstitutionDetail/>
                        <Route path=path!("") view=NoInstitution/>
                    </ParentRoute>
                    <ParentRoute path=path!("/home/transactions") view=Transactions>
                        <Route path=path!(":id") view=TransactionDetail/>
                        <Route path=path!("") view=NoTransaction/>
                    </ParentRoute>
                </Routes>
            </main>
        </Router>
    }
}

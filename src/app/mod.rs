use leptos::prelude::*;
use leptos_meta::{MetaTags, provide_meta_context};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    path,
};

use crate::app::{
    accounts::{AccountDetail, Accounts, NoAccount},
    assets::{AssetDetail, Assets, NoAsset},
    auth::{HandleAuth, Login},
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
pub struct RefreshToken(pub RwSignal<Option<String>>);

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let auth_token = RwSignal::<Option<String>, _>::new(None);
    let refresh_token = RwSignal::<Option<String>, _>::new(None);
    provide_context(AuthToken(auth_token));
    provide_context(RefreshToken(refresh_token));

    view! {
        <Router>
            <nav>
            <p>"Hi"</p>
            </nav>

            <Login/>

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

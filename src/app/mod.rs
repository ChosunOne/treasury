use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    path,
};

use crate::app::{
    accounts::{AccountDetail, Accounts, NoAccount},
    assets::{AssetDetail, Assets, NoAsset},
    home::Home,
    institutions::{InstitutionDetail, Institutions, NoInstitution},
    transactions::{NoTransaction, TransactionDetail, Transactions},
    users::{NoUser, UserDetail, Users},
};

pub mod accounts;
pub mod assets;
pub mod home;
pub mod institutions;
pub mod transactions;
pub mod users;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <nav>
            <p>"Hi"</p>
            </nav>

            <main>
                <Routes fallback=|| "This page could not be found.">
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

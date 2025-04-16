use futures::{Sink, Stream};
use leptos::{
    prelude::*,
    server_fn::{
        client::{Client, browser::BrowserClient},
        error::FromServerFnError,
        request::browser::BrowserRequest,
        response::browser::BrowserResponse,
    },
};

use crate::app::AuthToken;

pub struct ApiClient;

impl<E> Client<E> for ApiClient
where
    E: FromServerFnError,
{
    type Request = BrowserRequest;
    type Response = BrowserResponse;

    fn send(req: Self::Request) -> impl Future<Output = Result<Self::Response, E>> + Send {
        let headers = req.headers();
        let auth_token = expect_context::<AuthToken>()
            .0
            .get_untracked()
            .expect("Missing auth token");
        headers.append("Authorization", &format!("Bearer {auth_token}"));
        BrowserClient::send(req)
    }

    fn open_websocket(
        path: &str,
    ) -> impl Future<
        Output = Result<
            (
                impl Stream<Item = Result<server_fn::Bytes, E>> + Send + 'static,
                impl Sink<Result<server_fn::Bytes, E>> + Send + 'static,
            ),
            E,
        >,
    > + Send {
        BrowserClient::open_websocket(path)
    }

    fn spawn(future: impl Future<Output = ()> + Send + 'static) {
        <BrowserClient as Client<E>>::spawn(future)
    }
}

use derive_more::{Display, From, FromStr};
use oauth2::CsrfToken as OauthCsrfToken;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Default, Clone, FromRow, FromStr, From, Serialize, Deserialize, Display)]
pub struct CsrfToken {
    pub token: String,
}

impl From<OauthCsrfToken> for CsrfToken {
    fn from(value: OauthCsrfToken) -> Self {
        Self {
            token: value.into_secret(),
        }
    }
}

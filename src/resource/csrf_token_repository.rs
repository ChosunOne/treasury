use sqlx::{PgTransaction, query_as};

use crate::{
    model::csrf_token::CsrfToken,
    resource::{CreateRepository, DeleteRepository, GetRepository, RepositoryError},
};

#[derive(Debug, Clone, Copy)]
pub struct CsrfTokenRepository;

impl GetRepository<String, CsrfToken> for CsrfTokenRepository {
    async fn get(
        &self,
        mut session: PgTransaction<'_>,
        id: String,
    ) -> Result<CsrfToken, RepositoryError> {
        let csrf_token = query_as!(
            CsrfToken,
            r#"
                SELECT * FROM csrf_token
                where token = $1
            "#,
            id
        )
        .fetch_one(&mut *session)
        .await?;
        Ok(csrf_token)
    }
}

impl CreateRepository<CsrfToken, CsrfToken> for CsrfTokenRepository {
    async fn create(
        &self,
        mut session: PgTransaction<'_>,
        create_model: CsrfToken,
    ) -> Result<CsrfToken, RepositoryError> {
        let new_token = query_as!(
            CsrfToken,
            r#"
                INSERT INTO csrf_token (token)
                VALUES ($1)
                RETURNING *
            "#,
            create_model.token
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(new_token)
    }
}

impl DeleteRepository<String, CsrfToken> for CsrfTokenRepository {
    async fn delete(
        &self,
        mut session: PgTransaction<'_>,
        id: String,
    ) -> Result<CsrfToken, RepositoryError> {
        let deleted_token = query_as!(
            CsrfToken,
            r#"
                DELETE FROM csrf_token
                WHERE token = $1
                RETURNING *
            "#,
            id
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(deleted_token)
    }
}

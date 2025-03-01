use sqlx::{PgTransaction, QueryBuilder, query_as};

use crate::{
    model::cursor_key::{CursorKey, CursorKeyCreate, CursorKeyFilter, CursorKeyId},
    resource::{CreateRepository, GetListRepository, GetRepository, RepositoryError},
};

const MAX_LIMIT: i64 = 100;

#[derive(Debug, Clone)]
pub struct CursorKeyRepository;

impl GetRepository<CursorKeyId, CursorKey> for CursorKeyRepository {
    async fn get(
        &self,
        mut session: PgTransaction<'_>,
        id: CursorKeyId,
    ) -> Result<CursorKey, RepositoryError> {
        let cursor_key = query_as!(
            CursorKey,
            r#"
            SELECT * FROM cursor_key 
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_one(&mut *session)
        .await?;
        Ok(cursor_key)
    }
}

impl GetListRepository<CursorKey, CursorKeyFilter> for CursorKeyRepository {
    async fn get_list(
        &self,
        mut session: PgTransaction<'_>,
        offset: Option<i64>,
        limit: Option<i64>,
        filter: CursorKeyFilter,
    ) -> Result<Vec<CursorKey>, RepositoryError> {
        let offset = offset.map(|x| x.max(0)).unwrap_or(0);
        let limit = limit.map(|x| x.clamp(1, MAX_LIMIT)).unwrap_or(MAX_LIMIT);
        let mut query = QueryBuilder::new(
            r#"
            SELECT * FROM cursor_key
        "#,
        );

        query.push(r#"WHERE"#);
        if let Some(expires_at) = filter.expires_at {
            query.push(r#"expires_at IS NULL OR expires_at > "#);
            query.push_bind(expires_at);
        }

        query.push(r#"OFFSET "#);
        query.push_bind(offset);
        query.push(r#"LIMIT "#);
        query.push_bind(limit);

        let cursor_keys = query
            .build_query_as::<CursorKey>()
            .fetch_all(&mut *session)
            .await?;

        Ok(cursor_keys)
    }
}

impl CreateRepository<CursorKeyCreate, CursorKey> for CursorKeyRepository {
    async fn create(
        &self,
        mut session: PgTransaction<'_>,
        create_model: CursorKeyCreate,
    ) -> Result<CursorKey, RepositoryError> {
        let new_cursor_key = query_as!(
            CursorKey,
            r#"
            INSERT INTO cursor_key (expires_at) 
            VALUES ($1)
            RETURNING *
            "#,
            create_model.expires_at,
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(new_cursor_key)
    }
}

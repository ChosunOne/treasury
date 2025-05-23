use sqlx::{PgTransaction, QueryBuilder, query_as};

use crate::model::Filter;
use crate::model::user::{User, UserCreate, UserFilter, UserId};
use crate::resource::{
    CreateRepository, DeleteRepository, GetListRepository, GetRepository, MAX_LIMIT,
    RepositoryError, UpdateRepository,
};

#[derive(Debug, Clone, Copy)]
pub struct UserRepository;

impl UserRepository {
    pub async fn get_by_iss_and_sub(
        &self,
        mut session: PgTransaction<'_>,
        iss: String,
        sub: String,
    ) -> Result<Option<User>, RepositoryError> {
        let user = query_as!(
            User,
            r#"
                SELECT * FROM "user"
                WHERE iss = $1 AND sub = $2
            "#,
            iss,
            sub
        )
        .fetch_optional(&mut *session)
        .await?;
        Ok(user)
    }
}

impl GetRepository<UserId, User> for UserRepository {
    async fn get(
        &self,
        mut session: PgTransaction<'_>,
        id: UserId,
    ) -> Result<User, RepositoryError> {
        let user = query_as!(
            User,
            r#"
                SELECT * FROM "user"
                WHERE id = $1
            "#,
            id.0,
        )
        .fetch_one(&mut *session)
        .await?;
        Ok(user)
    }
}

impl GetListRepository<User, UserFilter> for UserRepository {
    async fn get_list(
        &self,
        mut session: PgTransaction<'_>,
        offset: i64,
        limit: Option<i64>,
        filter: UserFilter,
    ) -> Result<Vec<User>, RepositoryError> {
        let offset = offset.max(0);
        let limit = limit.map(|x| x.clamp(1, MAX_LIMIT)).unwrap_or(MAX_LIMIT);
        let mut query = QueryBuilder::new(
            r#"
            SELECT * FROM "user"
            "#,
        );

        filter.push(&mut query);

        query.push(r#" OFFSET "#);
        query.push_bind(offset);
        query.push(r#" LIMIT "#);
        query.push_bind(limit);

        let users = query
            .build_query_as::<User>()
            .fetch_all(&mut *session)
            .await?;

        Ok(users)
    }
}

impl CreateRepository<UserCreate, User> for UserRepository {
    async fn create(
        &self,
        mut session: PgTransaction<'_>,
        create_model: UserCreate,
    ) -> Result<User, RepositoryError> {
        let new_user = query_as!(
            User,
            r#"
                INSERT INTO "user" (name, email, iss, sub) 
                VALUES ($1, $2, $3, $4) 
                RETURNING *
            "#,
            create_model.name,
            create_model.email,
            create_model.iss,
            create_model.sub,
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(new_user)
    }
}

impl UpdateRepository<User> for UserRepository {
    async fn update(
        &self,
        mut session: PgTransaction<'_>,
        model: User,
    ) -> Result<User, RepositoryError> {
        let updated_user = query_as!(
            User,
            r#"
                UPDATE "user"
                SET name = $2, email = $3
                WHERE id = $1
                RETURNING *
            "#,
            model.id.0,
            model.name,
            model.email,
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(updated_user)
    }
}

impl DeleteRepository<UserId, User> for UserRepository {
    async fn delete(
        &self,
        mut session: PgTransaction<'_>,
        id: UserId,
    ) -> Result<User, RepositoryError> {
        let deleted_user = query_as!(
            User,
            r#"
                DELETE FROM "user"
                WHERE id = $1
                RETURNING *
            "#,
            id.0
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(deleted_user)
    }
}

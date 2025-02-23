use sqlx::{PgPool, QueryBuilder, query_as};

use crate::model::user::{User, UserCreate, UserFilter, UserId};
use crate::resource::{
    CreateRepository, DeleteRepository, GetListRepository, GetRepository, Repository,
    RepositoryError, UpdateRepository,
};

const MAX_LIMIT: i64 = 100;

#[derive(Debug, Clone)]
pub struct UserRepository {
    connection: PgPool,
}

impl UserRepository {
    pub fn new(connection: PgPool) -> Self {
        Self { connection }
    }
}

impl GetRepository<UserId, User> for UserRepository {
    async fn get(&self, id: UserId) -> Result<User, RepositoryError> {
        let user = query_as!(
            User,
            r#"
            SELECT * FROM "user" 
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_one(&self.connection)
        .await?;
        Ok(user)
    }
}

impl GetListRepository<User, UserFilter> for UserRepository {
    async fn get_list(
        &self,
        offset: i64,
        limit: i64,
        filter: Option<UserFilter>,
    ) -> Result<Vec<User>, RepositoryError> {
        let limit = limit.clamp(1, MAX_LIMIT);
        let offset = offset.max(0);
        let mut query = QueryBuilder::new(
            r#"
            SELECT * FROM "user"
            "#,
        );

        if let Some(f) = filter {
            query.push(r#"WHERE"#);
            let name_and_email = f.name.is_some() && f.email.is_some();

            if let Some(name) = f.name {
                query.push(r#"name = "#);
                query.push_bind(name);
            }
            if name_and_email {
                query.push("AND");
            }
            if let Some(email) = f.email {
                query.push(r#"email = "#);
                query.push_bind(email);
            }
        }

        query.push(r#"OFFSET "#);
        query.push_bind(offset);
        query.push(r#"LIMIT "#);
        query.push_bind(limit);

        let users = query
            .build_query_as::<User>()
            .fetch_all(&self.connection)
            .await?;

        Ok(users)
    }
}

impl CreateRepository<UserCreate, User> for UserRepository {
    async fn create(&self, create_model: UserCreate) -> Result<User, RepositoryError> {
        let new_user = query_as!(
            User,
            r#"
            INSERT INTO "user" (name, email) 
            VALUES ($1, $2) RETURNING *
            "#,
            create_model.name,
            create_model.email
        )
        .fetch_one(&self.connection)
        .await?;
        Ok(new_user)
    }
}

impl UpdateRepository<User> for UserRepository {
    async fn update(&self, model: User) -> Result<User, RepositoryError> {
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
        .fetch_one(&self.connection)
        .await?;
        Ok(updated_user)
    }
}

impl DeleteRepository<UserId, User> for UserRepository {
    async fn delete(&self, id: UserId) -> Result<User, RepositoryError> {
        let deleted_user = query_as!(
            User,
            r#"
            DELETE FROM "user"
            WHERE id = $1
            RETURNING *
            "#,
            id.0
        )
        .fetch_one(&self.connection)
        .await?;
        Ok(deleted_user)
    }
}

impl Repository<UserId, User, UserCreate, UserFilter> for UserRepository {}

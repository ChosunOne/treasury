use sqlx::{PgTransaction, QueryBuilder, query_as};

use crate::{
    model::{
        Filter,
        account::{Account, AccountCreate, AccountFilter, AccountId},
    },
    resource::{
        CreateRepository, DeleteRepository, GetListRepository, GetRepository, MAX_LIMIT,
        RepositoryError, UpdateRepository,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct AccountRepository;

impl GetRepository<AccountId, Account> for AccountRepository {
    async fn get(
        &self,
        mut session: PgTransaction<'_>,
        id: AccountId,
    ) -> Result<Account, RepositoryError> {
        let account = query_as!(
            Account,
            r#"
            SELECT * FROM account
            WHERE id = $1
        "#,
            id.0
        )
        .fetch_one(&mut *session)
        .await?;
        Ok(account)
    }
}

impl GetListRepository<Account, AccountFilter> for AccountRepository {
    async fn get_list(
        &self,
        mut session: PgTransaction<'_>,
        offset: i64,
        limit: Option<i64>,
        filter: AccountFilter,
    ) -> Result<Vec<Account>, RepositoryError> {
        let offset = offset.max(0);
        let limit = limit.map(|x| x.clamp(1, MAX_LIMIT)).unwrap_or(MAX_LIMIT);

        let mut query = QueryBuilder::new(
            r#"
            SELECT * from account
            "#,
        );

        filter.push(&mut query);
        query.push(r#" OFFSET "#);
        query.push_bind(offset);
        query.push(r#" LIMIT "#);
        query.push_bind(limit);

        let accounts = query
            .build_query_as::<Account>()
            .fetch_all(&mut *session)
            .await?;

        Ok(accounts)
    }
}

impl CreateRepository<AccountCreate, Account> for AccountRepository {
    async fn create(
        &self,
        mut session: PgTransaction<'_>,
        create_model: AccountCreate,
    ) -> Result<Account, RepositoryError> {
        let new_account = query_as!(
            Account,
            r#"
            INSERT INTO account (name, institution_id, user_id)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            create_model.name,
            create_model.institution_id.0,
            create_model.user_id.0,
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(new_account)
    }
}

impl UpdateRepository<Account> for AccountRepository {
    async fn update(
        &self,
        mut session: PgTransaction<'_>,
        model: Account,
    ) -> Result<Account, RepositoryError> {
        let updated_account = query_as!(
            Account,
            r#"
            UPDATE account
            SET name = $2, institution_id = $3, user_id = $4
            WHERE id = $1
            RETURNING *
            "#,
            model.id.0,
            model.name,
            model.institution_id.0,
            model.user_id.0,
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(updated_account)
    }
}

impl DeleteRepository<AccountId, Account> for AccountRepository {
    async fn delete(
        &self,
        mut session: PgTransaction<'_>,
        id: AccountId,
    ) -> Result<Account, RepositoryError> {
        let deleted_account = query_as!(
            Account,
            r#"
            DELETE FROM account
            WHERE id = $1
            RETURNING *
            "#,
            id.0
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(deleted_account)
    }
}

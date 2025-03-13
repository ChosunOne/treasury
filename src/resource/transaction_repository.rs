use sqlx::{PgTransaction, QueryBuilder, query_as};

use crate::{
    model::{
        Filter,
        transaction::{Transaction, TransactionCreate, TransactionFilter, TransactionId},
        user::UserId,
    },
    resource::{
        CreateRepository, DeleteRepository, GetListRepository, GetRepository, MAX_LIMIT,
        RepositoryError, UpdateRepository,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct TransactionRepository;

impl GetRepository<TransactionId, Transaction> for TransactionRepository {
    async fn get(
        &self,
        mut session: PgTransaction<'_>,
        id: TransactionId,
    ) -> Result<Transaction, RepositoryError> {
        let transaction = query_as!(
            Transaction,
            r#"
                SELECT * from "transaction"
                WHERE id = $1
            "#,
            id.0
        )
        .fetch_one(&mut *session)
        .await?;
        Ok(transaction)
    }
}

impl GetListRepository<Transaction, TransactionFilter> for TransactionRepository {
    async fn get_list(
        &self,
        mut session: PgTransaction<'_>,
        offset: i64,
        limit: Option<i64>,
        filter: TransactionFilter,
    ) -> Result<Vec<Transaction>, RepositoryError> {
        let offset = offset.max(0);
        let limit = limit.map(|x| x.clamp(1, MAX_LIMIT)).unwrap_or(MAX_LIMIT);
        let mut query = QueryBuilder::new(
            r#"
            SELECT * FROM "transaction"
            "#,
        );

        filter.push(&mut query);
        query.push(r#" OFFSET "#);
        query.push_bind(offset);
        query.push(r#" LIMIT "#);
        query.push_bind(limit);

        let transactions = query
            .build_query_as::<Transaction>()
            .fetch_all(&mut *session)
            .await?;
        Ok(transactions)
    }
}

impl CreateRepository<TransactionCreate, Transaction> for TransactionRepository {
    async fn create(
        &self,
        mut session: PgTransaction<'_>,
        create_model: TransactionCreate,
    ) -> Result<Transaction, RepositoryError> {
        let new_transaction = query_as!(
            Transaction,
            r#"
            INSERT INTO "transaction" (account_id, asset_id, description, posted_at, quantity)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            create_model.account_id.0,
            create_model.asset_id.0,
            create_model.description,
            create_model.posted_at,
            create_model.quantity
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(new_transaction)
    }
}

impl UpdateRepository<Transaction> for TransactionRepository {
    async fn update(
        &self,
        mut session: PgTransaction<'_>,
        model: Transaction,
    ) -> Result<Transaction, RepositoryError> {
        let updated_transaction = query_as!(
            Transaction,
            r#"
            UPDATE "transaction"
            SET account_id = $2, asset_id = $3, description = $4, posted_at = $5, quantity = $6
            WHERE id = $1
            RETURNING *
        "#,
            model.id.0,
            model.account_id.0,
            model.asset_id.0,
            model.description,
            model.posted_at,
            model.quantity,
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(updated_transaction)
    }
}

impl DeleteRepository<TransactionId, Transaction> for TransactionRepository {
    async fn delete(
        &self,
        mut session: PgTransaction<'_>,
        id: TransactionId,
    ) -> Result<Transaction, RepositoryError> {
        let deleted_transaction = query_as!(
            Transaction,
            r#"
                DELETE FROM "transaction"
                WHERE id = $1
                RETURNING *
            "#,
            id.0
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(deleted_transaction)
    }
}

impl TransactionRepository {
    pub async fn get_with_user_id(
        &self,
        mut session: PgTransaction<'_>,
        transaction_id: TransactionId,
        user_id: UserId,
    ) -> Result<Transaction, RepositoryError> {
        let transaction = query_as!(
            Transaction,
            r#"
            SELECT t.*
            FROM "transaction" t
            JOIN account a ON t.account_id = a.id
            WHERE t.id = $1
            AND a.user_id = $2
        "#,
            transaction_id.0,
            user_id.0
        )
        .fetch_one(&mut *session)
        .await?;
        Ok(transaction)
    }

    pub async fn get_list_with_user_id(
        &self,
        mut session: PgTransaction<'_>,
        offset: i64,
        limit: Option<i64>,
        user_id: UserId,
        filter: TransactionFilter,
    ) -> Result<Vec<Transaction>, RepositoryError> {
        let offset = offset.max(0);
        let limit = limit.map(|x| x.clamp(1, MAX_LIMIT)).unwrap_or(MAX_LIMIT);
        let mut query = QueryBuilder::new(
            r#"
            SELECT t.*
            FROM "transaction" t
            WHERE t.account_id IN (
                SELECT id
                FROM account
                WHERE user_id ="#,
        );
        query.push_bind(user_id);
        query.push(r#")"#);

        if let Some(description) = filter.description {
            query.push(r#" AND "#);
            query.push(r#"t.description ILIKE "#);
            query.push_bind(format!("%{description}%"));
        }

        if let Some(asset_id) = filter.asset_id {
            query.push(r#" AND "#);
            query.push(r#"t.asset_id = "#);
            query.push_bind(asset_id);
        }

        if let Some(account_id) = filter.account_id {
            query.push(r#" AND "#);
            query.push(r#"t.account_id = "#);
            query.push_bind(account_id);
        }

        if let Some(quantity) = filter.quantity {
            query.push(r#" AND "#);
            query.push(r#"t.quantity = "#);
            query.push_bind(quantity);
        }

        if let Some(max_quantity) = filter.max_quantity {
            query.push(r#" AND "#);
            query.push(r#"t.quantity <= "#);
            query.push_bind(max_quantity);
        }

        if let Some(min_quantity) = filter.min_quantity {
            query.push(r#" AND "#);
            query.push(r#"t.quantity >= "#);
            query.push_bind(min_quantity);
        }

        if let Some(posted_at) = filter.posted_at {
            query.push(r#" AND "#);
            query.push(r#"t.posted_at = "#);
            query.push_bind(posted_at);
        }

        if let Some(posted_before) = filter.posted_before {
            query.push(r#" AND "#);
            query.push(r#"t.posted_at < "#);
            query.push_bind(posted_before);
        }

        if let Some(posted_after) = filter.posted_after {
            query.push(r#" AND "#);
            query.push(r#"t.posted_at > "#);
            query.push_bind(posted_after);
        }

        query.push(r#" OFFSET "#);
        query.push_bind(offset);
        query.push(r#" LIMIT "#);
        query.push_bind(limit);

        let transactions = query
            .build_query_as::<Transaction>()
            .fetch_all(&mut *session)
            .await?;
        Ok(transactions)
    }

    pub async fn create_with_user_id(
        &self,
        mut session: PgTransaction<'_>,
        create_model: TransactionCreate,
        user_id: UserId,
    ) -> Result<Transaction, RepositoryError> {
        let transaction = query_as!(
            Transaction,
            r#"
            INSERT INTO "transaction" (account_id, asset_id, description, posted_at, quantity)
            SELECT $1, $2, $3, $4, $5
            WHERE EXISTS (
                SELECT 1
                FROM account
                WHERE id = $1
                AND user_id = $6
            )
            RETURNING *
        "#,
            create_model.account_id.0,
            create_model.asset_id.0,
            create_model.description,
            create_model.posted_at,
            create_model.quantity,
            user_id.0
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;

        Ok(transaction)
    }

    pub async fn update_with_user_id(
        &self,
        mut session: PgTransaction<'_>,
        model: Transaction,
        user_id: UserId,
    ) -> Result<Transaction, RepositoryError> {
        let transaction = query_as!(
            Transaction,
            r#"
                UPDATE "transaction"
                SET
                    asset_id = $1,
                    description = $2,
                    posted_at = $3,
                    quantity = $4
                WHERE
                    id = $5
                    AND account_id IN (
                        SELECT id
                        FROM account
                        WHERE
                            user_id = $6
                    )
                RETURNING *
        "#,
            model.asset_id.0,
            model.description,
            model.posted_at,
            model.quantity,
            model.id.0,
            user_id.0
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(transaction)
    }

    pub async fn delete_with_user_id(
        &self,
        mut session: PgTransaction<'_>,
        id: TransactionId,
        user_id: UserId,
    ) -> Result<Transaction, RepositoryError> {
        let deleted_transaction = query_as!(
            Transaction,
            r#"
                DELETE FROM "transaction"
                WHERE id = $1
                AND account_id IN (
                    SELECT id
                    FROM account
                    WHERE user_id = $2
                )
                RETURNING *
            "#,
            id.0,
            user_id.0,
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(deleted_transaction)
    }
}

use sqlx::{PgTransaction, QueryBuilder, query_as};

use crate::{
    model::{
        Filter,
        asset::{Asset, AssetCreate, AssetFilter, AssetId},
    },
    resource::{
        CreateRepository, DeleteRepository, GetListRepository, GetRepository, MAX_LIMIT,
        RepositoryError, UpdateRepository,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct AssetRepository;

impl GetRepository<AssetId, Asset> for AssetRepository {
    async fn get(
        &self,
        mut session: PgTransaction<'_>,
        id: AssetId,
    ) -> Result<Asset, RepositoryError> {
        let asset = query_as!(
            Asset,
            r#"
                SELECT * FROM asset
                WHERE id = $1
            "#,
            id.0
        )
        .fetch_one(&mut *session)
        .await?;
        Ok(asset)
    }
}

impl GetListRepository<Asset, AssetFilter> for AssetRepository {
    async fn get_list(
        &self,
        mut session: PgTransaction<'_>,
        offset: i64,
        limit: Option<i64>,
        filter: AssetFilter,
    ) -> Result<Vec<Asset>, RepositoryError> {
        let offset = offset.max(0);
        let limit = limit.map(|x| x.clamp(1, MAX_LIMIT)).unwrap_or(MAX_LIMIT);

        let mut query = QueryBuilder::new(
            r#"
            SELECT * FROM asset
            "#,
        );

        filter.push(&mut query);
        query.push(r#" OFFSET "#);
        query.push_bind(offset);
        query.push(r#" LIMIT "#);
        query.push_bind(limit);

        let assets = query
            .build_query_as::<Asset>()
            .fetch_all(&mut *session)
            .await?;
        Ok(assets)
    }
}

impl CreateRepository<AssetCreate, Asset> for AssetRepository {
    async fn create(
        &self,
        mut session: PgTransaction<'_>,
        create_model: AssetCreate,
    ) -> Result<Asset, RepositoryError> {
        let new_asset = query_as!(
            Asset,
            r#"
                INSERT INTO asset (name, symbol)
                VALUES ($1, $2)
                RETURNING *
            "#,
            create_model.name,
            create_model.symbol
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(new_asset)
    }
}

impl UpdateRepository<Asset> for AssetRepository {
    async fn update(
        &self,
        mut session: PgTransaction<'_>,
        model: Asset,
    ) -> Result<Asset, RepositoryError> {
        let updated_asset = query_as!(
            Asset,
            r#"
                UPDATE asset
                SET name = $2, symbol = $3
                WHERE id = $1
                RETURNING *
            "#,
            model.id.0,
            model.name,
            model.symbol
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(updated_asset)
    }
}

impl DeleteRepository<AssetId, Asset> for AssetRepository {
    async fn delete(
        &self,
        mut session: PgTransaction<'_>,
        id: AssetId,
    ) -> Result<Asset, RepositoryError> {
        let deleted_asset = query_as!(
            Asset,
            r#"
                DELETE FROM asset
                WHERE id = $1
                RETURNING *
            "#,
            id.0
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(deleted_asset)
    }
}

use sqlx::{PgTransaction, QueryBuilder, query_as};

use crate::{
    model::institution::{Institution, InstitutionCreate, InstitutionFilter, InstitutionId},
    resource::{
        CreateRepository, DeleteRepository, GetListRepository, GetRepository, MAX_LIMIT,
        Repository, RepositoryError, UpdateRepository,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct InstitutionRepository;

impl GetRepository<InstitutionId, Institution> for InstitutionRepository {
    async fn get(
        &self,
        mut session: PgTransaction<'_>,
        id: InstitutionId,
    ) -> Result<Institution, RepositoryError> {
        let institution = query_as!(
            Institution,
            r#"
            SELECT * FROM institution
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_one(&mut *session)
        .await?;
        Ok(institution)
    }
}

impl GetListRepository<Institution, InstitutionFilter> for InstitutionRepository {
    async fn get_list(
        &self,
        mut session: PgTransaction<'_>,
        offset: i64,
        limit: Option<i64>,
        filter: InstitutionFilter,
    ) -> Result<Vec<Institution>, RepositoryError> {
        let offset = offset.max(0);
        let limit = limit.map(|x| x.clamp(1, MAX_LIMIT)).unwrap_or(MAX_LIMIT);
        let mut query = QueryBuilder::new(
            r#"
            SELECT * from institution
            "#,
        );
        if filter.name.is_some() {
            query.push(r#"WHERE "#);
        }
        if let Some(name) = filter.name {
            query.push(r#"name = "#);
            query.push_bind(name);
        }

        query.push(r#" OFFSET "#);
        query.push_bind(offset);
        query.push(r#" LIMIT "#);
        query.push_bind(limit);

        let institutions = query
            .build_query_as::<Institution>()
            .fetch_all(&mut *session)
            .await?;
        Ok(institutions)
    }
}

impl CreateRepository<InstitutionCreate, Institution> for InstitutionRepository {
    async fn create(
        &self,
        mut session: PgTransaction<'_>,
        create_model: InstitutionCreate,
    ) -> Result<Institution, RepositoryError> {
        let new_institution = query_as!(
            Institution,
            r#"
            INSERT INTO institution (name)
            VALUES ($1)
            RETURNING *
            "#,
            create_model.name
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(new_institution)
    }
}

impl UpdateRepository<Institution> for InstitutionRepository {
    async fn update(
        &self,
        mut session: PgTransaction<'_>,
        model: Institution,
    ) -> Result<Institution, RepositoryError> {
        let updated_institution = query_as!(
            Institution,
            r#"
            UPDATE institution
            SET name = $2
            WHERE id = $1
            RETURNING *
            "#,
            model.id.0,
            model.name,
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(updated_institution)
    }
}

impl DeleteRepository<InstitutionId, Institution> for InstitutionRepository {
    async fn delete(
        &self,
        mut session: PgTransaction<'_>,
        id: InstitutionId,
    ) -> Result<Institution, RepositoryError> {
        let deleted_institution = query_as!(
            Institution,
            r#"
            DELETE FROM institution
            WHERE id = $1
            RETURNING *
            "#,
            id.0
        )
        .fetch_one(&mut *session)
        .await?;
        session.commit().await?;
        Ok(deleted_institution)
    }
}

impl Repository<InstitutionId, Institution, InstitutionCreate, InstitutionFilter>
    for InstitutionRepository
{
}

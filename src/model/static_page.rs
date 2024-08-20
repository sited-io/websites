use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::Pool;
use sea_query::{all, Asterisk, Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;
use serde_json::Value;

use crate::db::DbError;

#[derive(Debug, Clone, Copy, Iden)]
#[iden(rename = "static_pages")]
pub enum StaticPageIden {
    Table,
    PageId,
    WebsiteId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Components,
}

#[derive(Debug, Clone)]
pub struct StaticPage {
    pub page_id: i64,
    pub website_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub components: Value,
}

impl StaticPage {
    pub async fn create(
        pool: &Pool,
        page_id: i64,
        website_id: &String,
        user_id: &String,
        components: Value,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(StaticPageIden::Table)
            .columns([
                StaticPageIden::PageId,
                StaticPageIden::WebsiteId,
                StaticPageIden::UserId,
                StaticPageIden::Components,
            ])
            .values([
                page_id.into(),
                website_id.into(),
                user_id.into(),
                components.into(),
            ])?
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn get(
        pool: &Pool,
        page_id: i64,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(StaticPageIden::Table)
            .cond_where(Expr::col(StaticPageIden::PageId).eq(page_id))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn update(
        pool: &Pool,
        page_id: i64,
        user_id: &String,
        components: Value,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::update();
            query.table(StaticPageIden::Table);

            query.value(StaticPageIden::Components, components);

            query
                .cond_where(all![
                    Expr::col(StaticPageIden::PageId).eq(page_id),
                    Expr::col(StaticPageIden::UserId).eq(user_id)
                ])
                .returning_all()
                .build_postgres(PostgresQueryBuilder)
        };

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn delete(
        pool: &Pool,
        page_id: i64,
        user_id: &String,
    ) -> Result<(), DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(StaticPageIden::Table)
            .cond_where(all![
                Expr::col(StaticPageIden::PageId).eq(page_id),
                Expr::col(StaticPageIden::UserId).eq(user_id)
            ])
            .build_postgres(PostgresQueryBuilder);

        conn.query(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl From<&Row> for StaticPage {
    fn from(row: &Row) -> Self {
        Self {
            page_id: row.get(StaticPageIden::PageId.to_string().as_str()),
            website_id: row.get(StaticPageIden::WebsiteId.to_string().as_str()),
            user_id: row.get(StaticPageIden::UserId.to_string().as_str()),
            created_at: row.get(StaticPageIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(StaticPageIden::UpdatedAt.to_string().as_str()),
            components: row
                .get(StaticPageIden::Components.to_string().as_str()),
        }
    }
}

impl From<Row> for StaticPage {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}

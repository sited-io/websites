use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::Pool;
use sea_query::{Asterisk, Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;

use crate::db::{get_count_from_rows, DbError};

#[derive(Debug, Clone, Copy, Iden)]
#[iden(rename = "websites")]
pub enum WebsiteIden {
    Table,
    WebsiteId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Name,
    Domain,
    ClientId,
    ZitadelAppId,
}

#[derive(Debug, Clone)]
pub struct Website {
    pub website_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub domain: String,
    pub client_id: String,
    pub zitadel_app_id: String,
}

impl Website {
    pub async fn create(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
        name: &String,
        domain: &String,
        client_id: &String,
        zitadel_app_id: &String,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(WebsiteIden::Table)
            .columns([
                WebsiteIden::WebsiteId,
                WebsiteIden::UserId,
                WebsiteIden::Name,
                WebsiteIden::Domain,
                WebsiteIden::ClientId,
                WebsiteIden::ZitadelAppId,
            ])
            .values([
                website_id.into(),
                user_id.into(),
                name.into(),
                domain.into(),
                client_id.into(),
                zitadel_app_id.into(),
            ])?
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn get(
        pool: &Pool,
        website_id: &String,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(WebsiteIden::Table)
            .cond_where(Expr::col(WebsiteIden::WebsiteId).eq(website_id))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn get_by_domain(
        pool: &Pool,
        domain: &String,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(WebsiteIden::Table)
            .cond_where(Expr::col(WebsiteIden::Domain).eq(domain))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn get_by_name(
        pool: &Pool,
        name: &String,
        user_id: Option<&String>,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::select();
            query
                .column(Asterisk)
                .from(WebsiteIden::Table)
                .cond_where(Expr::col(WebsiteIden::Name).eq(name));

            if let Some(user_id) = user_id {
                query.cond_where(Expr::col(WebsiteIden::UserId).eq(user_id));
            }

            query.build_postgres(PostgresQueryBuilder)
        };

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn list(
        pool: &Pool,
        user_id: &Option<String>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Self>, i64), DbError> {
        let mut conn = pool.get().await?;
        let transaction = conn.transaction().await?;

        let ((sql, values), (count_sql, count_values)) = {
            let mut query = Query::select().from(WebsiteIden::Table).to_owned();

            if let Some(user_id) = user_id {
                query.cond_where(Expr::col(WebsiteIden::UserId).eq(user_id));
            }

            (
                query
                    .clone()
                    .column(Asterisk)
                    .limit(limit)
                    .offset(offset)
                    .build_postgres(PostgresQueryBuilder),
                query
                    .expr(Expr::col(Asterisk).count())
                    .build_postgres(PostgresQueryBuilder),
            )
        };

        let rows = transaction.query(sql.as_str(), &values.as_params()).await?;
        let count_rows = transaction
            .query(count_sql.as_str(), &count_values.as_params())
            .await?;
        transaction.commit().await?;

        let count = get_count_from_rows(&count_rows);

        Ok((rows.iter().map(Self::from).collect(), count))
    }

    pub async fn update(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
        name: &Option<String>,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::update();
            query.table(WebsiteIden::Table);

            if let Some(name) = name {
                query.value(WebsiteIden::Name, name);
            }

            query
                .and_where(Expr::col(WebsiteIden::WebsiteId).eq(website_id))
                .and_where(Expr::col(WebsiteIden::UserId).eq(user_id))
                .returning_all()
                .build_postgres(PostgresQueryBuilder)
        };

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn delete(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(WebsiteIden::Table)
            .and_where(Expr::col(WebsiteIden::WebsiteId).eq(website_id))
            .and_where(Expr::col(WebsiteIden::UserId).eq(user_id))
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }
}

impl From<&Row> for Website {
    fn from(row: &Row) -> Self {
        Self {
            website_id: row.get(WebsiteIden::WebsiteId.to_string().as_str()),
            user_id: row.get(WebsiteIden::UserId.to_string().as_str()),
            created_at: row.get(WebsiteIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(WebsiteIden::UpdatedAt.to_string().as_str()),
            name: row.get(WebsiteIden::Name.to_string().as_str()),
            domain: row.get(WebsiteIden::Domain.to_string().as_str()),
            client_id: row.get(WebsiteIden::ClientId.to_string().as_str()),
            zitadel_app_id: row
                .get(WebsiteIden::ZitadelAppId.to_string().as_str()),
        }
    }
}

impl From<Row> for Website {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}

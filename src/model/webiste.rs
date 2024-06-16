use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::Pool;
use sea_query::{
    all, Alias, Asterisk, Expr, Iden, PostgresQueryBuilder, Query,
    SelectStatement,
};
use sea_query_postgres::PostgresBinder;

use crate::db::{get_count_from_rows, DbError};

use super::domain::{DomainAsRel, DomainAsRelVec, DomainIden};
use super::page::PageAsRelVec;
use super::{CustomizationAsRel, PageAsRel};

#[derive(Debug, Clone, Copy, Iden)]
#[iden(rename = "websites")]
pub enum WebsiteIden {
    Table,
    WebsiteId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Name,
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
    pub client_id: String,
    pub zitadel_app_id: String,
    pub customization: Option<CustomizationAsRel>,
    pub domains: Vec<DomainAsRel>,
    pub pages: Vec<PageAsRel>,
}

impl Website {
    const DOMAINS_ALIAS: &'static str = "domains";
    const PAGES_ALIAS: &'static str = "pages";

    fn get_domains_alias() -> Alias {
        Alias::new(Self::DOMAINS_ALIAS)
    }

    fn get_pages_alias() -> Alias {
        Alias::new(Self::PAGES_ALIAS)
    }

    fn select_with_relations() -> SelectStatement {
        let mut query = Query::select();

        query
            .column((WebsiteIden::Table, Asterisk))
            .from(WebsiteIden::Table);

        CustomizationAsRel::add_join(&mut query);
        DomainAsRel::add_join(&mut query, Self::get_domains_alias());
        PageAsRel::add_join(&mut query, Self::get_pages_alias());

        query
    }

    fn select_count() -> SelectStatement {
        let mut query = Query::select();

        query
            .expr(Expr::col(Asterisk).count())
            .from(WebsiteIden::Table);

        query
    }

    pub async fn create(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
        name: &String,
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
                WebsiteIden::ClientId,
                WebsiteIden::ZitadelAppId,
            ])
            .values([
                website_id.into(),
                user_id.into(),
                name.into(),
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

        let (sql, values) = Self::select_with_relations()
            .cond_where(
                Expr::col((WebsiteIden::Table, WebsiteIden::WebsiteId))
                    .eq(website_id),
            )
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn get_for_user(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Self::select_with_relations()
            .cond_where(all![
                Expr::col((WebsiteIden::Table, WebsiteIden::WebsiteId))
                    .eq(website_id),
                Expr::col((WebsiteIden::Table, WebsiteIden::UserId))
                    .eq(user_id)
            ])
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn get_by_domain(
        pool: &Pool,
        domain: &String,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Self::select_with_relations()
            .cond_where(
                Expr::col((DomainIden::Table, DomainIden::Domain)).eq(domain),
            )
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
            let mut query = Self::select_with_relations();
            query.cond_where(
                Expr::col((WebsiteIden::Table, WebsiteIden::Name)).eq(name),
            );

            if let Some(user_id) = user_id {
                query.cond_where(
                    Expr::col((WebsiteIden::Table, WebsiteIden::UserId))
                        .eq(user_id),
                );
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
            let mut query = Self::select_with_relations();
            let mut count_query = Self::select_count();

            if let Some(user_id) = user_id {
                let where_user_id =
                    Expr::col((WebsiteIden::Table, WebsiteIden::UserId))
                        .eq(user_id);
                query.cond_where(where_user_id.clone());
                count_query.cond_where(where_user_id);
            }

            (
                query
                    .limit(limit)
                    .offset(offset)
                    .build_postgres(PostgresQueryBuilder),
                count_query.build_postgres(PostgresQueryBuilder),
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
                .cond_where(all![
                    Expr::col(WebsiteIden::WebsiteId).eq(website_id),
                    Expr::col(WebsiteIden::UserId).eq(user_id)
                ])
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
        let customization = CustomizationAsRel::try_from(row).ok();
        let domains = row
            .try_get::<&str, DomainAsRelVec>(Self::DOMAINS_ALIAS)
            .ok()
            .map(|d| d.0)
            .unwrap_or_default();
        let pages = row
            .try_get::<&str, PageAsRelVec>(Self::PAGES_ALIAS)
            .ok()
            .map(|p| p.0)
            .unwrap_or_default();

        Self {
            website_id: row.get(WebsiteIden::WebsiteId.to_string().as_str()),
            user_id: row.get(WebsiteIden::UserId.to_string().as_str()),
            created_at: row.get(WebsiteIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(WebsiteIden::UpdatedAt.to_string().as_str()),
            name: row.get(WebsiteIden::Name.to_string().as_str()),
            client_id: row.get(WebsiteIden::ClientId.to_string().as_str()),
            zitadel_app_id: row
                .get(WebsiteIden::ZitadelAppId.to_string().as_str()),
            customization,
            domains,
            pages,
        }
    }
}

impl From<Row> for Website {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}

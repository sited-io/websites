use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::types::{private, FromSql, Type};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::Pool;
use fallible_iterator::FallibleIterator;
use postgres_protocol::types;
use sea_query::{
    all, Alias, Asterisk, Expr, Func, Iden, JoinType, PostgresQueryBuilder,
    Query, SelectStatement,
};
use sea_query_postgres::PostgresBinder;

use crate::db::{get_count_from_rows, get_type_from_oid, ArrayAgg, DbError};

use super::webiste::WebsiteIden;

#[derive(Debug, Clone, Copy, Iden)]
#[iden(rename = "pages")]
pub enum PageIden {
    Table,
    PageId,
    WebsiteId,
    UserId,
    CreatedAt,
    UpdatedAt,
    PageType,
    ContentId,
    Title,
    IsHomePage,
    Path,
}

#[derive(Debug, Clone)]
pub struct Page {
    pub page_id: i64,
    pub website_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub page_type: String,
    pub content_id: String,
    pub title: String,
    pub is_home_page: bool,
    pub path: String,
}

impl Page {
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
        page_type: &str,
        content_id: &String,
        title: &String,
        is_home_page: bool,
        path: &String,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(PageIden::Table)
            .columns([
                PageIden::WebsiteId,
                PageIden::UserId,
                PageIden::PageType,
                PageIden::ContentId,
                PageIden::Title,
                PageIden::IsHomePage,
                PageIden::Path,
            ])
            .values([
                website_id.into(),
                user_id.into(),
                page_type.into(),
                content_id.into(),
                title.into(),
                is_home_page.into(),
                path.into(),
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
            .from(PageIden::Table)
            .cond_where(Expr::col(PageIden::PageId).eq(page_id))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn get_by_path(
        pool: &Pool,
        website_id: &String,
        path: &String,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(PageIden::Table)
            .cond_where(all![
                Expr::col(PageIden::WebsiteId).eq(website_id),
                Expr::col(PageIden::Path).eq(path)
            ])
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn get_home_page(
        pool: &Pool,
        website_id: &String,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(PageIden::Table)
            .cond_where(all![
                Expr::col(PageIden::WebsiteId).eq(website_id),
                Expr::col(PageIden::IsHomePage).eq(true)
            ])
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn list(
        pool: &Pool,
        website_id: Option<String>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Self>, i64), DbError> {
        let conn = pool.get().await?;

        let ((sql, values), (count_sql, count_values)) = {
            let mut query = Query::select();

            query.from(PageIden::Table);

            if let Some(website_id) = website_id {
                query.cond_where(Expr::col(PageIden::WebsiteId).eq(website_id));
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

        let rows = conn.query(sql.as_str(), &values.as_params()).await?;
        let count_rows = conn
            .query(count_sql.as_str(), &count_values.as_params())
            .await?;

        let count = get_count_from_rows(&count_rows);

        Ok((rows.iter().map(Self::from).collect(), count))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        pool: &Pool,
        page_id: i64,
        user_id: &String,
        page_type: Option<&str>,
        content_id: Option<String>,
        title: Option<String>,
        is_home_page: Option<bool>,
        path: Option<String>,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = {
            let mut query = Query::update();
            query.table(PageIden::Table);

            if let Some(page_type) = page_type {
                query.value(PageIden::PageType, page_type);
            }

            if let Some(content_id) = content_id {
                query.value(PageIden::ContentId, content_id);
            }

            if let Some(title) = title {
                query.value(PageIden::Title, title);
            }

            if let Some(is_home_page) = is_home_page {
                query.value(PageIden::IsHomePage, is_home_page);
            }

            if let Some(path) = path {
                query.value(PageIden::Path, path);
            }

            query
                .cond_where(all![
                    Expr::col(PageIden::PageId).eq(page_id),
                    Expr::col(PageIden::UserId).eq(user_id)
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
            .from_table(PageIden::Table)
            .cond_where(all![
                Expr::col(PageIden::PageId).eq(page_id),
                Expr::col(PageIden::UserId).eq(user_id)
            ])
            .build_postgres(PostgresQueryBuilder);

        conn.query(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }

    pub async fn delete_for_website(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
    ) -> Result<(), DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(PageIden::Table)
            .cond_where(all![
                Expr::col(WebsiteIden::WebsiteId).eq(website_id),
                Expr::col(WebsiteIden::UserId).eq(user_id)
            ])
            .build_postgres(PostgresQueryBuilder);

        conn.query(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl From<&Row> for Page {
    fn from(row: &Row) -> Self {
        Self {
            page_id: row.get(PageIden::PageId.to_string().as_str()),
            website_id: row.get(PageIden::WebsiteId.to_string().as_str()),
            user_id: row.get(PageIden::UserId.to_string().as_str()),
            created_at: row.get(PageIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(PageIden::UpdatedAt.to_string().as_str()),
            page_type: row.get(PageIden::PageType.to_string().as_str()),
            content_id: row.get(PageIden::ContentId.to_string().as_str()),
            title: row.get(PageIden::Title.to_string().as_str()),
            is_home_page: row.get(PageIden::IsHomePage.to_string().as_str()),
            path: row.get(PageIden::Path.to_string().as_str()),
        }
    }
}

impl From<Row> for Page {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}

#[derive(Debug, Clone)]
pub struct PageAsRel {
    pub page_id: i64,
    pub page_type: String,
    pub content_id: String,
    pub title: String,
    pub is_home_page: bool,
    pub path: String,
}

impl PageAsRel {
    pub fn add_join(query: &mut SelectStatement, alias: Alias) {
        query
            .column((PageIden::Table, alias.clone()))
            .join_subquery(
                JoinType::LeftJoin,
                Query::select()
                    .column(PageIden::WebsiteId)
                    .expr_as(
                        Func::cust(ArrayAgg).args([Expr::tuple([
                            Expr::col((PageIden::Table, PageIden::PageId))
                                .into(),
                            Expr::col((PageIden::Table, PageIden::PageType))
                                .into(),
                            Expr::col((PageIden::Table, PageIden::ContentId))
                                .into(),
                            Expr::col((PageIden::Table, PageIden::Title))
                                .into(),
                            Expr::col((PageIden::Table, PageIden::IsHomePage))
                                .into(),
                            Expr::col((PageIden::Table, PageIden::Path)).into(),
                        ])
                        .into()]),
                        alias.clone(),
                    )
                    .from(PageIden::Table)
                    .group_by_col(PageIden::WebsiteId)
                    .take(),
                alias.clone(),
                Expr::col((WebsiteIden::Table, WebsiteIden::WebsiteId))
                    .equals((PageIden::Table, PageIden::WebsiteId)),
            )
            .group_by_col((PageIden::Table, alias));
    }
}

impl<'a> FromSql<'a> for PageAsRel {
    fn accepts(ty: &Type) -> bool {
        matches!(*ty, Type::RECORD)
    }

    fn from_sql(
        _: &Type,
        mut raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        private::read_be_i32(&mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<i64>(oid)?;
        let page_id: i64 = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<String>(oid)?;
        let page_type: String = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<String>(oid)?;
        let content_id: String = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<String>(oid)?;
        let title: String = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<bool>(oid)?;
        let is_home_page: bool = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<String>(oid)?;
        let path: String = private::read_value(&ty, &mut raw)?;

        Ok(Self {
            page_id,
            page_type,
            content_id,
            title,
            is_home_page,
            path,
        })
    }
}

impl From<Page> for PageAsRel {
    fn from(page: Page) -> Self {
        Self {
            page_id: page.page_id,
            page_type: page.page_type,
            content_id: page.content_id,
            title: page.title,
            is_home_page: page.is_home_page,
            path: page.path,
        }
    }
}

pub struct PageAsRelVec(pub Vec<PageAsRel>);

impl<'a> FromSql<'a> for PageAsRelVec {
    fn accepts(ty: &Type) -> bool {
        matches!(*ty, Type::RECORD_ARRAY)
    }

    fn from_sql(
        _: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let array = types::array_from_sql(raw)?;

        if array.dimensions().count()? > 1 {
            return Err("[PageAsRelVec::FromSql::from_sql]: array contains too many dimensions".into());
        }

        Ok(Self(
            array
                .values()
                .filter_map(|v| {
                    Ok(PageAsRel::from_sql_nullable(&Type::RECORD, v).ok())
                })
                .collect()?,
        ))
    }
}

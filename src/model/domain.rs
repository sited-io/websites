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

use crate::db::{get_type_from_oid, ArrayAgg, DbError};

use super::webiste::WebsiteIden;

#[derive(Debug, Clone, Copy, Iden)]
#[iden(rename = "domains")]
pub enum DomainIden {
    Table,
    DomainId,
    WebsiteId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Domain,
    Status,
}

#[derive(Debug, Clone)]
pub struct Domain {
    pub domain_id: i64,
    pub website_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub domain: String,
    pub status: String,
}

impl Domain {
    pub async fn create(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
        domain: &String,
        status: &'static str,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(DomainIden::Table)
            .columns([
                DomainIden::WebsiteId,
                DomainIden::UserId,
                DomainIden::Domain,
                DomainIden::Status,
            ])
            .values([
                website_id.into(),
                user_id.into(),
                domain.into(),
                status.into(),
            ])?
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn get_for_user(
        pool: &Pool,
        domain_id: i64,
        user_id: &String,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(DomainIden::Table)
            .cond_where(all![
                Expr::col(DomainIden::DomainId).eq(domain_id),
                Expr::col(DomainIden::UserId).eq(user_id)
            ])
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn get_for_website(
        pool: &Pool,
        domain: &String,
        website_id: &String,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(DomainIden::Table)
            .cond_where(all![
                Expr::col(DomainIden::Domain).eq(domain),
                Expr::col(DomainIden::WebsiteId).eq(website_id)
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

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(DomainIden::Table)
            .cond_where(Expr::col(DomainIden::Domain).eq(domain))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn get_by_domain_and_status(
        pool: &Pool,
        domain: &String,
        status: &'static str,
    ) -> Result<Option<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(DomainIden::Table)
            .cond_where(all![
                Expr::col(DomainIden::Domain).eq(domain),
                Expr::col(DomainIden::Status).eq(status)
            ])
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn list_by_status(
        pool: &Pool,
        status: &'static str,
    ) -> Result<Vec<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(DomainIden::Table)
            .cond_where(Expr::col(DomainIden::Status).eq(status))
            .build_postgres(PostgresQueryBuilder);

        let rows = conn.query(sql.as_str(), &values.as_params()).await?;

        Ok(rows.iter().map(Self::from).collect())
    }

    pub async fn update(
        pool: &Pool,
        domain_id: i64,
        website_id: &String,
        user_id: &String,
        status: &'static str,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::update()
            .table(DomainIden::Table)
            .value(DomainIden::Status, status)
            .cond_where(all![
                Expr::col(DomainIden::DomainId).eq(domain_id),
                Expr::col(DomainIden::WebsiteId).eq(website_id),
                Expr::col(DomainIden::UserId).eq(user_id),
            ])
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn delete_for_website(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
    ) -> Result<(), DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(DomainIden::Table)
            .cond_where(all![
                Expr::col(DomainIden::WebsiteId).eq(website_id),
                Expr::col(DomainIden::UserId).eq(user_id)
            ])
            .build_postgres(PostgresQueryBuilder);

        conn.query(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }

    pub async fn delete(
        pool: &Pool,
        domain_id: i64,
        website_id: &String,
        user_id: &String,
    ) -> Result<(), DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(DomainIden::Table)
            .cond_where(all![
                Expr::col(DomainIden::DomainId).eq(domain_id),
                Expr::col(DomainIden::WebsiteId).eq(website_id),
                Expr::col(DomainIden::UserId).eq(user_id),
            ])
            .build_postgres(PostgresQueryBuilder);

        conn.query(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl From<&Row> for Domain {
    fn from(row: &Row) -> Self {
        Self {
            domain_id: row.get(DomainIden::DomainId.to_string().as_str()),
            website_id: row.get(DomainIden::WebsiteId.to_string().as_str()),
            user_id: row.get(DomainIden::UserId.to_string().as_str()),
            created_at: row.get(DomainIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(DomainIden::UpdatedAt.to_string().as_str()),
            domain: row.get(DomainIden::Domain.to_string().as_str()),
            status: row.get(DomainIden::Status.to_string().as_str()),
        }
    }
}

impl From<Row> for Domain {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}

#[derive(Debug, Clone)]
pub struct DomainAsRel {
    pub domain_id: i64,
    pub domain: String,
    pub status: String,
}

impl DomainAsRel {
    pub fn add_join(query: &mut SelectStatement, alias: Alias) {
        query
            .column((DomainIden::Table, alias.clone()))
            .join_subquery(
                JoinType::LeftJoin,
                Query::select()
                    .column(DomainIden::WebsiteId)
                    .expr_as(
                        Func::cust(ArrayAgg).args([Expr::tuple([
                            Expr::col((
                                DomainIden::Table,
                                DomainIden::DomainId,
                            ))
                            .into(),
                            Expr::col((DomainIden::Table, DomainIden::Domain))
                                .into(),
                            Expr::col((DomainIden::Table, DomainIden::Status))
                                .into(),
                        ])
                        .into()]),
                        alias.clone(),
                    )
                    .from(DomainIden::Table)
                    .group_by_col(DomainIden::WebsiteId)
                    .take(),
                alias.clone(),
                Expr::col((WebsiteIden::Table, WebsiteIden::WebsiteId))
                    .equals((DomainIden::Table, DomainIden::WebsiteId)),
            )
            .group_by_col((DomainIden::Table, alias));
    }
}

impl<'a> FromSql<'a> for DomainAsRel {
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
        let domain_id: i64 = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<String>(oid)?;
        let domain: String = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<String>(oid)?;
        let status: String = private::read_value(&ty, &mut raw)?;

        Ok(Self {
            domain_id,
            domain,
            status,
        })
    }
}

impl From<Domain> for DomainAsRel {
    fn from(domain: Domain) -> Self {
        Self {
            domain_id: domain.domain_id,
            domain: domain.domain,
            status: domain.status,
        }
    }
}

pub struct DomainAsRelVec(pub Vec<DomainAsRel>);

impl<'a> FromSql<'a> for DomainAsRelVec {
    fn accepts(ty: &Type) -> bool {
        match *ty {
            Type::RECORD_ARRAY => true,
            _ => {
                tracing::log::error!("[DomainAsRelVec::FromSql::accepts]: postgres type {:?} not implemented", ty);
                false
            }
        }
    }

    fn from_sql(
        _: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let array = types::array_from_sql(raw)?;

        if array.dimensions().count()? > 1 {
            return Err("[DomainAsRelVec::FromSql::from_sql]: array contains too many dimensions".into());
        }

        Ok(Self(
            array
                .values()
                .filter_map(|v| {
                    Ok(DomainAsRel::from_sql_nullable(&Type::RECORD, v).ok())
                })
                .collect()?,
        ))
    }
}

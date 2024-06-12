use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres::types::{private, FromSql, Type};
use deadpool_postgres::tokio_postgres::Row;
use deadpool_postgres::Pool;
use fallible_iterator::FallibleIterator;
use postgres_protocol::types;
use sea_query::{
    all, Asterisk, Expr, Func, Iden, PostgresQueryBuilder, Query, SimpleExpr,
};
use sea_query_postgres::PostgresBinder;

use crate::db::{get_type_from_oid, ArrayAgg, DbError};

#[derive(Debug, Clone, Copy, Iden)]
#[iden(rename = "domains")]
pub enum DomainIden {
    Table,
    WebsiteId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Domain,
    CloudflareDnsRecordId,
}

#[derive(Debug, Clone)]
pub struct Domain {
    pub domain: String,
    pub website_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cloudflare_dns_record_id: Option<String>,
}

impl Domain {
    pub async fn create(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
        domain: &String,
        cloudflare_dns_record_id: &Option<String>,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(DomainIden::Table)
            .columns([
                DomainIden::WebsiteId,
                DomainIden::UserId,
                DomainIden::Domain,
                DomainIden::CloudflareDnsRecordId,
            ])
            .values([
                website_id.into(),
                user_id.into(),
                domain.into(),
                cloudflare_dns_record_id.to_owned().into(),
            ])?
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn list_for_website(
        pool: &Pool,
        website_id: &String,
    ) -> Result<Vec<Self>, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(DomainIden::Table)
            .cond_where(Expr::col(DomainIden::WebsiteId).eq(website_id))
            .build_postgres(PostgresQueryBuilder);

        let rows = conn.query(sql.as_str(), &values.as_params()).await?;

        Ok(rows.iter().map(Self::from).collect())
    }

    pub async fn delete(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
        domain: &String,
    ) -> Result<(), DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(DomainIden::Table)
            .cond_where(all![
                Expr::col(DomainIden::WebsiteId).eq(website_id),
                Expr::col(DomainIden::UserId).eq(user_id),
                Expr::col(DomainIden::Domain).eq(domain),
            ])
            .build_postgres(PostgresQueryBuilder);

        conn.query(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl From<&Row> for Domain {
    fn from(row: &Row) -> Self {
        Self {
            domain: row.get(DomainIden::Domain.to_string().as_str()),
            website_id: row.get(DomainIden::WebsiteId.to_string().as_str()),
            user_id: row.get(DomainIden::UserId.to_string().as_str()),
            created_at: row.get(DomainIden::CreatedAt.to_string().as_str()),
            updated_at: row.get(DomainIden::UpdatedAt.to_string().as_str()),
            cloudflare_dns_record_id: row
                .get(DomainIden::CloudflareDnsRecordId.to_string().as_str()),
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
    pub domain: String,
    pub cloudflare_dns_record_id: Option<String>,
}

impl DomainAsRel {
    pub fn get_agg() -> SimpleExpr {
        Func::cust(ArrayAgg)
            .args([Expr::tuple([
                Expr::col((DomainIden::Table, DomainIden::Domain)).into(),
                Expr::col((
                    DomainIden::Table,
                    DomainIden::CloudflareDnsRecordId,
                ))
                .into(),
            ])
            .into()])
            .into()
    }
}

impl<'a> FromSql<'a> for DomainAsRel {
    fn accepts(ty: &deadpool_postgres::tokio_postgres::types::Type) -> bool {
        matches!(*ty, Type::RECORD)
    }

    fn from_sql(
        _: &Type,
        mut raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        private::read_be_i32(&mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<String>(oid)?;
        let domain: String = private::read_value(&ty, &mut raw)?;

        let oid = private::read_be_i32(&mut raw)?;
        let ty = get_type_from_oid::<Option<String>>(oid)?;
        let cloudflare_dns_record_id: Option<String> =
            private::read_value(&ty, &mut raw)?;

        Ok(Self {
            domain,
            cloudflare_dns_record_id,
        })
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

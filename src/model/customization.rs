use deadpool_postgres::tokio_postgres::{Error, Row};
use deadpool_postgres::Pool;
use sea_query::{
    all, Asterisk, Expr, Iden, PostgresQueryBuilder, Query, SelectStatement,
};
use sea_query_postgres::PostgresBinder;

use crate::db::DbError;

use super::webiste::WebsiteIden;

#[derive(Debug, Clone, Copy, Iden)]
#[iden(rename = "customizations")]
pub enum CustomizationIden {
    Table,
    WebsiteId,
    UserId,
    PrimaryColor,
    SecondaryColor,
    LogoImageUrl,
}

#[derive(Debug, Clone)]
pub struct Customization {
    pub website_id: String,
    pub user_id: String,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub logo_image_url: Option<String>,
}

impl Customization {
    pub async fn create(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::insert()
            .into_table(CustomizationIden::Table)
            .columns([CustomizationIden::WebsiteId, CustomizationIden::UserId])
            .values([website_id.into(), user_id.into()])?
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
            .from(CustomizationIden::Table)
            .cond_where(Expr::col(CustomizationIden::WebsiteId).eq(website_id))
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_opt(sql.as_str(), &values.as_params()).await?;

        Ok(row.map(Self::from))
    }

    pub async fn update(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
        primary_color: Option<String>,
        secondary_color: Option<String>,
        logo_image_url: Option<String>,
    ) -> Result<Self, DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::update()
            .table(CustomizationIden::Table)
            .values([
                (CustomizationIden::PrimaryColor, primary_color.into()),
                (CustomizationIden::SecondaryColor, secondary_color.into()),
                (CustomizationIden::LogoImageUrl, logo_image_url.into()),
            ])
            .cond_where(all![
                Expr::col(CustomizationIden::WebsiteId).eq(website_id),
                Expr::col(CustomizationIden::UserId).eq(user_id)
            ])
            .returning_all()
            .build_postgres(PostgresQueryBuilder);

        let row = conn.query_one(sql.as_str(), &values.as_params()).await?;

        Ok(Self::from(row))
    }

    pub async fn delete(
        pool: &Pool,
        website_id: &String,
        user_id: &String,
    ) -> Result<(), DbError> {
        let conn = pool.get().await?;

        let (sql, values) = Query::delete()
            .from_table(CustomizationIden::Table)
            .cond_where(all![
                Expr::col(CustomizationIden::WebsiteId).eq(website_id),
                Expr::col(CustomizationIden::UserId).eq(user_id)
            ])
            .build_postgres(PostgresQueryBuilder);

        conn.query(sql.as_str(), &values.as_params()).await?;

        Ok(())
    }
}

impl From<&Row> for Customization {
    fn from(row: &Row) -> Self {
        Self {
            website_id: row
                .get(CustomizationIden::WebsiteId.to_string().as_str()),
            user_id: row.get(CustomizationIden::UserId.to_string().as_str()),
            primary_color: row
                .get(CustomizationIden::PrimaryColor.to_string().as_str()),
            secondary_color: row
                .get(CustomizationIden::SecondaryColor.to_string().as_str()),
            logo_image_url: row
                .get(CustomizationIden::LogoImageUrl.to_string().as_str()),
        }
    }
}

impl From<Row> for Customization {
    fn from(row: Row) -> Self {
        Self::from(&row)
    }
}

#[derive(Debug, Clone)]
pub struct CustomizationAsRel {
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub logo_image_url: Option<String>,
}

impl CustomizationAsRel {
    pub fn add_join(query: &mut SelectStatement) {
        query
            .columns([
                (CustomizationIden::Table, CustomizationIden::PrimaryColor),
                (CustomizationIden::Table, CustomizationIden::SecondaryColor),
                (CustomizationIden::Table, CustomizationIden::LogoImageUrl),
            ])
            .left_join(
                CustomizationIden::Table,
                Expr::col((WebsiteIden::Table, WebsiteIden::WebsiteId)).equals(
                    (CustomizationIden::Table, CustomizationIden::WebsiteId),
                ),
            )
            .group_by_columns([
                (CustomizationIden::Table, CustomizationIden::PrimaryColor),
                (CustomizationIden::Table, CustomizationIden::SecondaryColor),
                (CustomizationIden::Table, CustomizationIden::LogoImageUrl),
            ]);
    }
}

impl TryFrom<&Row> for CustomizationAsRel {
    type Error = Error;
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            primary_color: row.try_get(
                CustomizationIden::PrimaryColor.to_string().as_str(),
            )?,
            secondary_color: row.try_get(
                CustomizationIden::SecondaryColor.to_string().as_str(),
            )?,
            logo_image_url: row.try_get(
                CustomizationIden::LogoImageUrl.to_string().as_str(),
            )?,
        })
    }
}

impl From<Customization> for CustomizationAsRel {
    fn from(customization: Customization) -> Self {
        Self {
            primary_color: customization.primary_color,
            secondary_color: customization.secondary_color,
            logo_image_url: customization.logo_image_url,
        }
    }
}

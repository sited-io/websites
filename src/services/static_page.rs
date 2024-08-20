use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};

use crate::api::sited_io::websites::v1::static_page_service_server::{
    self, StaticPageServiceServer,
};
use crate::api::sited_io::websites::v1::{
    GetStaticPageRequest, GetStaticPageResponse, StaticPageResponse,
    UpdateStaticPageRequest, UpdateStaticPageResponse,
};
use crate::auth::get_user_id;
use crate::model::StaticPage;

pub struct StaticPageService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
}

impl StaticPageService {
    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
    ) -> StaticPageServiceServer<Self> {
        StaticPageServiceServer::new(Self { pool, verifier })
    }

    fn to_response(static_page: StaticPage) -> StaticPageResponse {
        StaticPageResponse {
            page_id: static_page.page_id,
            website_id: static_page.website_id,
            user_id: static_page.user_id,
            components: serde_json::from_value(static_page.components).unwrap(),
        }
    }
}

#[async_trait]
impl static_page_service_server::StaticPageService for StaticPageService {
    async fn get_static_page(
        &self,
        request: Request<GetStaticPageRequest>,
    ) -> Result<Response<GetStaticPageResponse>, Status> {
        let GetStaticPageRequest { page_id } = request.into_inner();

        let found_static_page = StaticPage::get(&self.pool, page_id).await?;

        Ok(Response::new(GetStaticPageResponse {
            static_page: found_static_page.map(Self::to_response),
        }))
    }

    async fn update_static_page(
        &self,
        request: Request<UpdateStaticPageRequest>,
    ) -> Result<Response<UpdateStaticPageResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let UpdateStaticPageRequest {
            page_id,
            components,
        } = request.into_inner();

        let updated_static_page = StaticPage::update(
            &self.pool,
            page_id,
            &user_id,
            serde_json::to_value(components).unwrap(),
        )
        .await?;

        Ok(Response::new(UpdateStaticPageResponse {
            static_page: Some(Self::to_response(updated_static_page)),
        }))
    }
}

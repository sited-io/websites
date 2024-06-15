use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};

use crate::api::sited_io::websites::v1::customization_service_server::{
    self, CustomizationServiceServer,
};
use crate::api::sited_io::websites::v1::{
    CustomizationResponse, PutCustomizationRequest, PutCustomizationResponse,
};
use crate::auth::get_user_id;
use crate::model::{Customization, CustomizationAsRel};

pub struct CustomizationService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
}

impl CustomizationService {
    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
    ) -> CustomizationServiceServer<Self> {
        CustomizationServiceServer::new(Self { pool, verifier })
    }

    pub fn to_response(
        customization: impl Into<CustomizationAsRel>,
    ) -> CustomizationResponse {
        let customization: CustomizationAsRel = customization.into();
        CustomizationResponse {
            primary_color: customization.primary_color,
            secondary_color: customization.secondary_color,
        }
    }
}

#[async_trait]
impl customization_service_server::CustomizationService
    for CustomizationService
{
    async fn put_customization(
        &self,
        request: Request<PutCustomizationRequest>,
    ) -> Result<Response<PutCustomizationResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let PutCustomizationRequest {
            website_id,
            primary_color,
            secondary_color,
        } = request.into_inner();

        let updated_customization = Customization::update(
            &self.pool,
            &website_id,
            &user_id,
            primary_color,
            secondary_color,
        )
        .await?;

        Ok(Response::new(PutCustomizationResponse {
            customization: Some(Self::to_response(updated_customization)),
        }))
    }
}

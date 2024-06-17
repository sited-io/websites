use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use crate::api::sited_io::websites::v1::customization_service_server::{
    self, CustomizationServiceServer,
};
use crate::api::sited_io::websites::v1::{
    CustomizationResponse, PutLogoImageRequest, PutLogoImageResponse,
    RemoveLogoImageRequest, RemoveLogoImageResponse,
    UpdateCustomizationRequest, UpdateCustomizationResponse,
};
use crate::auth::get_user_id;
use crate::images::ImageService;
use crate::model::{Customization, CustomizationAsRel};

pub struct CustomizationService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
    image_service: ImageService,
}

impl CustomizationService {
    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
        image_service: ImageService,
    ) -> CustomizationServiceServer<Self> {
        CustomizationServiceServer::new(Self {
            pool,
            verifier,
            image_service,
        })
    }

    pub fn to_response(
        image_service: &ImageService,
        customization: impl Into<CustomizationAsRel>,
    ) -> CustomizationResponse {
        let customization: CustomizationAsRel = customization.into();
        CustomizationResponse {
            primary_color: customization.primary_color,
            secondary_color: customization.secondary_color,
            logo_image_url: image_service
                .get_opt_image_url(customization.logo_image_url),
        }
    }

    fn gen_image_path(user_id: &String, website_id: &String) -> String {
        format!("{}/{}/{}", user_id, website_id, Uuid::new_v4())
    }
}

#[async_trait]
impl customization_service_server::CustomizationService
    for CustomizationService
{
    async fn update_customization(
        &self,
        request: Request<UpdateCustomizationRequest>,
    ) -> Result<Response<UpdateCustomizationResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let UpdateCustomizationRequest {
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

        Ok(Response::new(UpdateCustomizationResponse {
            customization: Some(Self::to_response(
                &self.image_service,
                updated_customization,
            )),
        }))
    }

    async fn put_logo_image(
        &self,
        request: Request<PutLogoImageRequest>,
    ) -> Result<Response<PutLogoImageResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let PutLogoImageRequest { website_id, image } = request.into_inner();

        let image = image.ok_or_else(|| {
            Status::invalid_argument("Please provide parameter image")
        })?;

        self.image_service.validate_image(&image.data)?;

        let existing_customization =
            Customization::get(&self.pool, &website_id).await?;

        if let Some(existing) = existing_customization
            .as_ref()
            .and_then(|c| c.logo_image_url.as_ref())
        {
            self.image_service.remove_image(existing).await?;
        }

        let image_path = Self::gen_image_path(&user_id, &website_id);

        self.image_service
            .put_image(&image_path, &image.data)
            .await?;

        Customization::update_logo_image(
            &self.pool,
            &website_id,
            &user_id,
            Some(image_path),
        )
        .await?;

        Ok(Response::new(PutLogoImageResponse {}))
    }

    async fn remove_logo_image(
        &self,
        request: Request<RemoveLogoImageRequest>,
    ) -> Result<Response<RemoveLogoImageResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let RemoveLogoImageRequest { website_id } = request.into_inner();

        let existing_customization =
            Customization::get(&self.pool, &website_id).await?;

        if let Some(existing) = existing_customization
            .as_ref()
            .and_then(|c| c.logo_image_url.as_ref())
        {
            self.image_service.remove_image(existing).await?;
        }

        Customization::update_logo_image(
            &self.pool,
            &website_id,
            &user_id,
            None,
        )
        .await?;

        Ok(Response::new(RemoveLogoImageResponse {}))
    }
}

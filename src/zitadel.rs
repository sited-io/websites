use tonic::transport::Channel;
use tonic::{Request, Response, Status};
use zitadel::api::zitadel::app::v1::{
    OidcAppType, OidcAuthMethodType, OidcGrantType, OidcResponseType,
};
use zitadel::api::zitadel::management::v1::management_service_client::ManagementServiceClient;
use zitadel::api::zitadel::management::v1::{
    AddOidcAppRequest, AddOidcAppResponse, GetAppByIdRequest,
    GetAppByIdResponse, RemoveAppRequest, RemoveAppResponse,
};
use zitadel::api::zitadel::user::v1::AccessTokenType;

#[derive(Debug, Clone)]
pub struct ZitadelService {
    management_service_client: ManagementServiceClient<Channel>,
    service_user_token: String,
    project_id: String,
}

impl ZitadelService {
    pub async fn init(
        url: String,
        service_user_token: String,
        project_id: String,
    ) -> Result<Self, tonic::transport::Error> {
        Ok(Self {
            management_service_client: ManagementServiceClient::connect(url)
                .await
                .unwrap(),
            service_user_token,
            project_id,
        })
    }

    pub async fn add_app(
        &mut self,
        name: String,
        redirect_uris: Vec<String>,
        post_logout_redirect_uris: Vec<String>,
    ) -> Result<Response<AddOidcAppResponse>, Status> {
        let mut req = Request::new(AddOidcAppRequest {
            project_id: self.project_id.clone(),
            name,
            redirect_uris,
            response_types: vec![OidcResponseType::Code.into()],
            grant_types: vec![
                OidcGrantType::AuthorizationCode.into(),
                OidcGrantType::RefreshToken.into(),
            ],
            app_type: OidcAppType::Web.into(),
            auth_method_type: OidcAuthMethodType::None.into(),
            post_logout_redirect_uris,
            access_token_type: AccessTokenType::Jwt.into(),
            ..Default::default()
        });
        req.metadata_mut()
            .insert("authorization", self.service_user_token.parse().unwrap());

        self.management_service_client.add_oidc_app(req).await
    }

    pub async fn get_app(
        &mut self,
        app_id: String,
    ) -> Result<Response<GetAppByIdResponse>, Status> {
        let mut req = Request::new(GetAppByIdRequest {
            project_id: self.project_id.clone(),
            app_id,
        });
        req.metadata_mut()
            .insert("authorization", self.service_user_token.parse().unwrap());
        self.management_service_client.get_app_by_id(req).await
    }

    pub async fn remove_app(
        &mut self,
        app_id: String,
    ) -> Result<Response<RemoveAppResponse>, Status> {
        let mut req = Request::new(RemoveAppRequest {
            project_id: self.project_id.clone(),
            app_id,
        });
        req.metadata_mut()
            .insert("authorization", self.service_user_token.parse().unwrap());
        self.management_service_client.remove_app(req).await
    }
}

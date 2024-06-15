use deadpool_postgres::Pool;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};
use zitadel::api::zitadel::management::v1::AddOidcAppResponse;

use crate::api::sited_io::websites::v1::website_service_server::WebsiteServiceServer;
use crate::api::sited_io::websites::v1::{
    website_service_server, CreateWebsiteRequest, CreateWebsiteResponse,
    DeleteWebsiteRequest, DeleteWebsiteResponse, DomainStatus,
    GetWebsiteRequest, GetWebsiteResponse, ListWebsitesRequest,
    ListWebsitesResponse, UpdateWebsiteRequest, UpdateWebsiteResponse,
    WebsiteResponse,
};
use crate::auth::get_user_id;
use crate::cloudflare::CloudflareService;
use crate::model::{Customization, Domain, Website};
use crate::zitadel::ZitadelService;
use crate::{
    datetime_to_timestamp, i64_to_u32, CustomizationService, DomainService,
};

use super::get_limit_offset_from_pagination;

pub struct WebsiteService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
    main_domain: String,
    fallback_domain: String,
    zitadel_service: ZitadelService,
    cloudflare_service: CloudflareService,
}

const WEBSITE_ID_LENGTH: usize = 14;

const MININUM_WEBSITE_NAME_LENGTH: usize = 4;

const DOMAIN_ALPHABET: [char; 36] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e',
    'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't',
    'u', 'v', 'w', 'x', 'y', 'z',
];

impl WebsiteService {
    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
        main_domain: String,
        fallback_domain: String,
        zitadel_service: ZitadelService,
        cloudflare_service: CloudflareService,
    ) -> WebsiteServiceServer<Self> {
        WebsiteServiceServer::new(Self {
            pool,
            verifier,
            main_domain,
            fallback_domain,
            zitadel_service,
            cloudflare_service,
        })
    }

    fn to_response(&self, website: Website) -> WebsiteResponse {
        WebsiteResponse {
            website_id: website.website_id.to_string(),
            user_id: website.user_id,
            created_at: datetime_to_timestamp(website.created_at),
            updated_at: datetime_to_timestamp(website.updated_at),
            name: website.name,
            client_id: website.client_id,
            customization: website
                .customization
                .map(CustomizationService::to_response),
            domains: website
                .domains
                .into_iter()
                .map(DomainService::to_response)
                .collect(),
        }
    }

    fn generate_website_id(&self) -> String {
        nanoid::nanoid!(WEBSITE_ID_LENGTH, &DOMAIN_ALPHABET)
    }

    fn build_main_domain(&self, website_id: &String) -> String {
        format!("{}.{}", website_id, self.main_domain)
    }
}

#[async_trait]
impl website_service_server::WebsiteService for WebsiteService {
    async fn create_website(
        &self,
        request: Request<CreateWebsiteRequest>,
    ) -> Result<Response<CreateWebsiteResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let CreateWebsiteRequest { name } = request.into_inner();

        if name.len() < MININUM_WEBSITE_NAME_LENGTH {
            return Err(Status::invalid_argument("name is too short"));
        }

        if Website::get_by_name(&self.pool, &name, Some(&user_id))
            .await?
            .is_some()
        {
            return Err(Status::invalid_argument("duplicate name"));
        }

        let website_id = self.generate_website_id();
        let domain = self.build_main_domain(&website_id);

        let mut zitadel_service = self.zitadel_service.clone();
        let redirect_uri = format!("https://{}/user/sign-in-callback", domain);
        let post_logout_redirect_uri = format!("https://{}", domain);

        let res = zitadel_service
            .add_app(
                domain.clone(),
                vec![redirect_uri],
                vec![post_logout_redirect_uri],
            )
            .await?;

        let AddOidcAppResponse {
            client_id, app_id, ..
        } = res.into_inner();

        self.cloudflare_service
            .create_dns_record(domain.clone(), self.fallback_domain.clone())
            .await?;

        let created_website = Website::create(
            &self.pool,
            &website_id,
            &user_id,
            &name,
            &client_id,
            &app_id,
        )
        .await?;

        Customization::create(&self.pool, &website_id, &user_id).await?;

        Domain::create(
            &self.pool,
            &website_id,
            &user_id,
            &domain,
            DomainStatus::Internal.as_str_name(),
        )
        .await?;

        Ok(Response::new(CreateWebsiteResponse {
            website: Some(self.to_response(created_website)),
        }))
    }

    async fn get_website(
        &self,
        request: Request<GetWebsiteRequest>,
    ) -> Result<Response<GetWebsiteResponse>, Status> {
        let GetWebsiteRequest {
            website_id, domain, ..
        } = request.into_inner();

        let found_website =
            match (website_id, domain) {
                (Some(website_id), _) => {
                    Website::get(&self.pool, &website_id).await?
                }
                (_, Some(domain)) => {
                    Website::get_by_domain(&self.pool, &domain).await?
                }
                _ => return Err(Status::invalid_argument(
                    "Please provide at least one of 'website_id' or 'domain'",
                )),
            };

        Ok(Response::new(GetWebsiteResponse {
            website: found_website.map(|w| self.to_response(w)),
        }))
    }

    async fn list_websites(
        &self,
        request: Request<ListWebsitesRequest>,
    ) -> Result<Response<ListWebsitesResponse>, Status> {
        let ListWebsitesRequest {
            user_id,
            pagination,
        } = request.into_inner();

        let (limit, offset, mut pagination) =
            get_limit_offset_from_pagination(pagination)?;

        let (found_websites, count) =
            Website::list(&self.pool, &user_id, limit, offset).await?;

        pagination.total_elements = i64_to_u32(count)?;

        Ok(Response::new(ListWebsitesResponse {
            webistes: found_websites
                .into_iter()
                .map(|w| self.to_response(w))
                .collect(),
            pagination: Some(pagination),
        }))
    }

    async fn update_website(
        &self,
        request: Request<UpdateWebsiteRequest>,
    ) -> Result<Response<UpdateWebsiteResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let UpdateWebsiteRequest { website_id, name } = request.into_inner();

        let updated_website =
            Website::update(&self.pool, &website_id, &user_id, &name).await?;

        Ok(Response::new(UpdateWebsiteResponse {
            website: Some(self.to_response(updated_website)),
        }))
    }

    async fn delete_website(
        &self,
        request: Request<DeleteWebsiteRequest>,
    ) -> Result<Response<DeleteWebsiteResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let DeleteWebsiteRequest { website_id } = request.into_inner();

        if let Some(found_website) =
            Website::get(&self.pool, &website_id).await?
        {
            let mut zitadel_service = self.zitadel_service.clone();
            zitadel_service
                .remove_app(found_website.zitadel_app_id)
                .await?;

            for domain in found_website.domains {
                let found_records = self
                    .cloudflare_service
                    .list_dns_records(Some(domain.domain.clone()))
                    .await?;
                for record in found_records.result {
                    self.cloudflare_service
                        .delete_dns_record(record.id)
                        .await?;
                }

                if domain.status == DomainStatus::Active.as_str_name() {
                    let found_custom_hostnames = self
                        .cloudflare_service
                        .list_custom_hostnames(&domain.domain)
                        .await?;
                    for custom_hostname in found_custom_hostnames.result {
                        self.cloudflare_service
                            .delete_custom_hostname(custom_hostname.id)
                            .await?;
                    }
                }
            }

            Customization::delete(&self.pool, &website_id, &user_id).await?;

            Domain::delete_for_website(&self.pool, &website_id, &user_id)
                .await?;

            Website::delete(&self.pool, &website_id, &user_id).await?;
        }

        Ok(Response::new(DeleteWebsiteResponse::default()))
    }
}

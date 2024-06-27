use std::collections::HashSet;

use deadpool_postgres::Pool;
use http::Uri;
use jwtk::jwk::RemoteJwksVerifier;
use tonic::{async_trait, Request, Response, Status};

use crate::api::sited_io::websites::v1::domain_service_server::{
    self, DomainServiceServer,
};
use crate::api::sited_io::websites::v1::{
    CheckDomainStatusRequest, CheckDomainStatusResponse, CreateDomainRequest,
    CreateDomainResponse, DeleteDomainRequest, DeleteDomainResponse,
    DomainResponse, DomainStatus,
};
use crate::auth::get_user_id;
use crate::cloudflare::{CloudflareService, DnsLookupResponse};
use crate::model::{Domain, DomainAsRel, Website};

pub struct DomainService {
    pool: Pool,
    verifier: RemoteJwksVerifier,
    fallback_domain: String,
    cloudflare_service: CloudflareService,
}

impl DomainService {
    pub fn build(
        pool: Pool,
        verifier: RemoteJwksVerifier,
        fallback_domain: String,
        cloudflare_service: CloudflareService,
    ) -> DomainServiceServer<Self> {
        DomainServiceServer::new(Self {
            pool,
            verifier,
            fallback_domain,
            cloudflare_service,
        })
    }

    pub fn to_response(domain: impl Into<DomainAsRel>) -> DomainResponse {
        let domain: DomainAsRel = domain.into();
        DomainResponse {
            domain_id: domain.domain_id,
            domain: domain.domain,
            status: DomainStatus::from_str_name(&domain.status).unwrap().into(),
        }
    }

    pub fn validate_domain(input: &String) -> Result<(), Status> {
        if !input.contains('.') {
            return Err(Status::invalid_argument(
                "Domain must contain a dot ('.')",
            ));
        }

        let parsed = input
            .parse::<Uri>()
            .map_err(|_| Status::invalid_argument("Domain is not valid"))?;

        let host = parsed
            .host()
            .ok_or_else(|| Status::invalid_argument("Domain is not valid"))?;

        if host != input {
            Err(Status::invalid_argument("Domain is not valid"))
        } else {
            Ok(())
        }
    }

    fn has_same_destination_ips(
        &self,
        a: &DnsLookupResponse,
        b: &DnsLookupResponse,
    ) -> bool {
        let a_ips: HashSet<&String> = a
            .answer
            .as_ref()
            .map(|a| a.iter().map(|a| &a.data).collect())
            .unwrap_or_default();
        let b_ips: HashSet<&String> = b
            .answer
            .as_ref()
            .map(|b| b.iter().map(|b| &b.data).collect())
            .unwrap_or_default();
        a_ips.intersection(&b_ips).eq(a_ips.iter())
    }

    fn has_cname_to_fallback(
        &self,
        domain: &String,
        dns_lookup_response: &DnsLookupResponse,
    ) -> bool {
        dns_lookup_response
            .answer
            .as_ref()
            .map(|answers| {
                answers.iter().find(|a| {
                    a.name == *domain
                        && a._type == 5
                        && a.data == self.fallback_domain
                })
            })
            .is_some()
    }
}

#[async_trait]
impl domain_service_server::DomainService for DomainService {
    async fn create_domain(
        &self,
        request: Request<CreateDomainRequest>,
    ) -> Result<Response<CreateDomainResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let CreateDomainRequest { website_id, domain } = request.into_inner();

        Self::validate_domain(&domain)?;

        if Website::get(&self.pool, &website_id)
            .await?
            .is_some_and(|w| w.user_id == user_id)
        {
            if Domain::get_by_domain_and_status(
                &self.pool,
                &domain,
                DomainStatus::Active.as_str_name(),
            )
            .await?
            .is_some()
            {
                return Err(Status::invalid_argument(
                    "Domain is already in use",
                ));
            };

            let created_domain = Domain::create(
                &self.pool,
                &website_id,
                &user_id,
                &domain,
                DomainStatus::Pending.as_str_name(),
            )
            .await?;

            Ok(Response::new(CreateDomainResponse {
                domain: Some(Self::to_response(created_domain)),
            }))
        } else {
            Err(Status::invalid_argument(format!(
                "Could not find website by website_id '{}'",
                website_id
            )))
        }
    }

    async fn check_domain_status(
        &self,
        request: Request<CheckDomainStatusRequest>,
    ) -> Result<Response<CheckDomainStatusResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let CheckDomainStatusRequest { domain_id } = request.into_inner();

        if let Some(mut domain) =
            Domain::get_for_user(&self.pool, domain_id, &user_id).await?
        {
            if domain.status == DomainStatus::Pending.as_str_name() {
                let domain_lookup =
                    self.cloudflare_service.dns_lookup(&domain.domain).await?;

                let mut points_to_fallback =
                    self.has_cname_to_fallback(&domain.domain, &domain_lookup);

                if !points_to_fallback {
                    let fallback_lookup = self
                        .cloudflare_service
                        .dns_lookup(&self.fallback_domain)
                        .await?;

                    points_to_fallback = self.has_same_destination_ips(
                        &fallback_lookup,
                        &domain_lookup,
                    );
                }

                if points_to_fallback {
                    self.cloudflare_service
                        .create_custom_hostname(domain.domain)
                        .await?;
                    domain = Domain::update(
                        &self.pool,
                        domain.domain_id,
                        &domain.website_id,
                        &domain.user_id,
                        DomainStatus::Active.as_str_name(),
                    )
                    .await?;
                }
            }

            Ok(Response::new(CheckDomainStatusResponse {
                domain: Some(Self::to_response(domain)),
            }))
        } else {
            Err(Status::invalid_argument(format!(
                "Could not find domain '{}'",
                domain_id
            )))
        }
    }

    async fn delete_domain(
        &self,
        request: Request<DeleteDomainRequest>,
    ) -> Result<Response<DeleteDomainResponse>, Status> {
        let user_id = get_user_id(request.metadata(), &self.verifier).await?;

        let DeleteDomainRequest { domain_id } = request.into_inner();

        if let Some(found_domain) =
            Domain::get_for_user(&self.pool, domain_id, &user_id).await?
        {
            if found_domain.status != DomainStatus::Internal.as_str_name() {
                let found_custom_hostnames = self
                    .cloudflare_service
                    .list_custom_hostnames(&found_domain.domain)
                    .await?;

                for custom_hostname in found_custom_hostnames.result {
                    self.cloudflare_service
                        .delete_custom_hostname(custom_hostname.id)
                        .await?;
                }

                Domain::delete(
                    &self.pool,
                    found_domain.domain_id,
                    &found_domain.website_id,
                    &user_id,
                )
                .await?;

                return Ok(Response::new(DeleteDomainResponse {}));
            }
        }

        Err(Status::invalid_argument(format!(
            "Could not find domain '{}'",
            domain_id
        )))
    }
}

use http::header::AUTHORIZATION;
use http::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tonic::Status;

#[derive(Debug, Serialize)]
struct CreateDnsRecordRequest {
    content: String,
    name: String,
    proxied: bool,
    #[serde(rename = "type")]
    _type: String,
    ttl: usize,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareResponses<B> {
    result: Vec<B>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct CloudflareResponse<B> {
    pub result: B,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct DnsRecordResponse {
    pub id: String,
    content: String,
    name: String,
    proxied: bool,
    #[serde(rename = "type")]
    _type: Option<String>,
    comment: Option<String>,
}

#[derive(Clone)]
pub struct CloudflareService {
    api_url: String,
    zone_id: String,
    main_domain: String,
    client: Client,
}

impl CloudflareService {
    pub fn init(
        api_url: String,
        zone_id: String,
        main_domain: String,
        token: String,
    ) -> Self {
        let mut default_headers = HeaderMap::with_capacity(1);
        default_headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(token.as_str()).unwrap(),
        );
        let client = Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();
        Self {
            api_url,
            zone_id,
            main_domain,
            client,
        }
    }

    pub async fn create_dns_record(
        &self,
        name: String,
    ) -> Result<CloudflareResponse<DnsRecordResponse>, Status> {
        let body = CreateDnsRecordRequest {
            name,
            content: self.main_domain.clone(),
            proxied: true,
            _type: "CNAME".to_string(),
            ttl: 1,
        };

        self.client
            .post(format!(
                "{}/zones/{}/dns_records",
                self.api_url, self.zone_id
            ))
            .json(&body)
            .send()
            .await
            .map_err(|err| {
                tracing::log::error!(
                    "[CloudflareService.create_dns_record]: {:?}",
                    err
                );
                Status::internal("")
            })?
            .json()
            .await
            .map_err(|err| {
                tracing::log::error!(
                    "[CloudflareService.create_dns_record]: {:?}",
                    err
                );
                Status::internal("")
            })
    }

    pub async fn list_dns_records(
        &self,
        name: Option<String>,
    ) -> Result<CloudflareResponses<DnsRecordResponse>, Status> {
        let mut req = self.client.get(format!(
            "{}/zones/{}/dns_records",
            self.api_url, self.zone_id
        ));

        if let Some(name) = name {
            req = req.query(&[("name", name)]);
        }

        req.send()
            .await
            .map_err(|err| {
                tracing::log::error!(
                    "[CloudflareService.list_dns_records]: {:?}",
                    err
                );
                Status::internal("")
            })?
            .json()
            .await
            .map_err(|err| {
                tracing::log::error!(
                    "[CloudflareService.list_dns_records]: {:?}",
                    err
                );
                Status::internal("")
            })
    }

    pub async fn delete_dns_record(
        &self,
        record_id: String,
    ) -> Result<(), Status> {
        self.client
            .delete(format!(
                "{}/zones/{}/dns_records/{}",
                self.api_url, self.zone_id, record_id
            ))
            .send()
            .await
            .map_err(|err| {
                tracing::log::error!(
                    "[CloudflareService.delete_dns_record]: {:?}",
                    err
                );
                Status::internal("")
            })?;

        Ok(())
    }
}

use std::collections::HashMap;

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

#[derive(Debug, Serialize)]
struct CreateCustomHostnameRequest {
    hostname: String,
    ssl: CreateCustomHostnameSslRequest,
}

#[derive(Debug, Serialize)]
struct CreateCustomHostnameSslRequest {
    method: &'static str,
    #[serde(rename = "type")]
    _type: &'static str,
    wildcard: bool,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareResponses<B> {
    pub result: Vec<B>,
    pub errors: Vec<HashMap<String, String>>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct CloudflareResponse<B> {
    pub result: B,
    pub errors: Vec<HashMap<String, String>>,
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

#[derive(Debug, Deserialize)]
pub struct CustomHostnameResponse {
    pub id: String,
    pub hostname: String,
}

#[derive(Debug, Deserialize)]
pub struct DnsLookupResponse {
    #[serde(rename = "Status")]
    pub status: usize,
    #[serde(rename = "Answer")]
    pub answer: Option<Vec<DnsLookupResponseAnswer>>,
    #[serde(rename = "Authority")]
    pub authority: Option<Vec<DnsLookupResponseAnswer>>,
    #[serde(rename = "Additional")]
    pub additional: Option<Vec<DnsLookupResponseAnswer>>,
}

#[derive(Debug, Deserialize)]
pub struct DnsLookupResponseAnswer {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: usize,
    #[serde(rename = "TTL")]
    pub ttl: usize,
    pub data: String,
}

const CLOUDFLARE_DNS_URL: &str = "https://cloudflare-dns.com/dns-query";

#[derive(Clone)]
pub struct CloudflareService {
    api_url: String,
    zone_id: String,
    client: Client,
}

impl CloudflareService {
    pub fn init(
        api_url: String,
        zone_id: String,
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
            client,
        }
    }

    pub async fn create_dns_record(
        &self,
        name: String,
        content: String,
    ) -> Result<CloudflareResponse<DnsRecordResponse>, Status> {
        let body = CreateDnsRecordRequest {
            name,
            content,
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

    pub async fn create_custom_hostname(
        &self,
        hostname: String,
    ) -> Result<CloudflareResponse<CustomHostnameResponse>, Status> {
        let body = CreateCustomHostnameRequest {
            hostname,
            ssl: CreateCustomHostnameSslRequest {
                method: "http",
                _type: "dv",
                wildcard: false,
            },
        };

        self.client
            .post(format!(
                "{}/zones/{}/custom_hostnames",
                self.api_url, self.zone_id
            ))
            .json(&body)
            .send()
            .await
            .map_err(|err| {
                tracing::log::error!(
                    "[CloudflareService.create_custom_hostname]: {:?}",
                    err
                );
                Status::internal("")
            })?
            .json()
            .await
            .map_err(|err| {
                tracing::log::error!(
                    "[CloudflareService.create_custom_hostname]: {:?}",
                    err
                );
                Status::internal("")
            })
    }

    pub async fn list_custom_hostnames(
        &self,
        hostname: &String,
    ) -> Result<CloudflareResponses<CustomHostnameResponse>, Status> {
        self.client
            .get(format!(
                "{}/zones/{}/custom_hostnames",
                self.api_url, self.zone_id
            ))
            .query(&[("hostname", hostname)])
            .send()
            .await
            .map_err(|err| {
                tracing::log::error!(
                    "[CloudflareService.list_custom_hostnames]: {:?}",
                    err
                );
                Status::internal("")
            })?
            .json()
            .await
            .map_err(|err| {
                tracing::log::error!(
                    "[CloudflareService.list_custom_hostnames]: {:?}",
                    err
                );
                Status::internal("")
            })
    }

    pub async fn delete_custom_hostname(
        &self,
        custom_hostname_id: String,
    ) -> Result<(), Status> {
        let url = format!(
            "{}/zones/{}/custom_hostnames/{}",
            self.api_url, self.zone_id, custom_hostname_id
        );

        self.client
            .delete(url)
            .send()
            .await
            .map_err(|err| {
                tracing::log::error!(
                    "[CloudflareService.delete_custom_hostname]: {:?}",
                    err
                );
                Status::internal("")
            })?
            .text()
            .await
            .unwrap();

        Ok(())
    }

    pub async fn dns_lookup(
        &self,
        domain: &String,
    ) -> Result<DnsLookupResponse, Status> {
        self.client
            .get(CLOUDFLARE_DNS_URL)
            .query(&[("name", domain)])
            .header("accept", "application/dns-json")
            .send()
            .await
            .map_err(|err| {
                tracing::log::error!(
                    "[CloudflareService.dns_lookup]: {:?}",
                    err
                );
                Status::internal("")
            })?
            .json()
            .await
            .map_err(|err| {
                tracing::log::error!(
                    "[CloudflareService.dns_lookup]: {:?}",
                    err
                );
                Status::internal("")
            })
    }
}

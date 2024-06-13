use deadpool_postgres::Pool;
use serde::Deserialize;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

use crate::api::sited_io::websites::v1::DomainStatus;
use crate::cloudflare::CloudflareService;
use crate::get_env_var;
use crate::model::Domain;

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct DnsLookupResponse {
    #[serde(rename = "Status")]
    status: usize,
    #[serde(rename = "Answer")]
    answer: Option<Vec<DnsLookupResponseAnswer>>,
    #[serde(rename = "Authority")]
    authority: Option<Vec<DnsLookupResponseAnswer>>,
    #[serde(rename = "Additional")]
    additional: Option<Vec<DnsLookupResponseAnswer>>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct DnsLookupResponseAnswer {
    name: String,
    #[serde(rename = "type")]
    _type: usize,
    #[serde(rename = "TTL")]
    ttl: usize,
    data: String,
}

const CLOUDFLARE_DNS_URL: &'static str = "https://cloudflare-dns.com/dns-query";

pub async fn run_custom_hostnames_check(
    pool: Pool,
    cloudflare_service: CloudflareService,
) -> Result<(), JobSchedulerError> {
    let sched = JobScheduler::new().await?;
    let main_domain = get_env_var("MAIN_DOMAIN");

    sched
        .add(Job::new_async("0 * * * * *", move |_, _| {
            let pool = pool.clone();
            let cloudflare_service = cloudflare_service.clone();
            let main_domain = main_domain.clone();

            Box::pin(async move {
                if let Err(err) =
                    check_custom_domains(pool, cloudflare_service, main_domain)
                        .await
                {
                    tracing::log::error!(
                        "[run_custom_hostnames_check]: {:?}",
                        err
                    );
                }
            })
        })?)
        .await?;

    sched.start().await?;

    Ok(())
}

async fn check_custom_domains(
    pool: Pool,
    cloudflare_service: CloudflareService,
    main_domain: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let domains =
        Domain::list_by_status(&pool, DomainStatus::Pending.as_str_name())
            .await?;

    let client = reqwest::Client::new();

    let mut root_ips = fetch_ips(&client, &main_domain).await?;
    root_ips.sort_unstable();

    for domain in domains {
        let mut domain_ips = fetch_ips(&client, &domain.domain).await?;
        domain_ips.sort_unstable();
        if root_ips == domain_ips {
            tracing::log::info!(
                "[run_custom_hostnames_check]: {} points to cloudflare.",
                domain.domain
            );
            cloudflare_service
                .create_custom_hostname(domain.domain.clone())
                .await?;

            Domain::update(
                &pool,
                domain.domain_id,
                &domain.website_id,
                &domain.user_id,
                DomainStatus::Active.as_str_name(),
            )
            .await?;
        }
    }

    Ok(())
}

async fn fetch_ips(
    client: &reqwest::Client,
    domain: &String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let res: DnsLookupResponse = client
        .get(CLOUDFLARE_DNS_URL)
        .query(&[("name", domain)])
        .header("accept", "application/dns-json")
        .send()
        .await?
        .json()
        .await?;

    if let Some(answers) = res.answer {
        Ok(answers.into_iter().map(|answer| answer.data).collect())
    } else {
        Ok(vec![])
    }
}

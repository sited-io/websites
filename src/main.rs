use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use http::{HeaderName, Method};
use tonic::transport::Server;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use websites::api::sited_io::websites::v1::website_service_server::WebsiteServiceServer;
use websites::cloudflare::CloudflareService;
use websites::custom_hostnames::run_custom_hostnames_check;
use websites::db::{init_db_pool, migrate};
use websites::logging::{LogOnFailure, LogOnRequest, LogOnResponse};
use websites::zitadel::ZitadelService;
use websites::{get_env_var, init_jwks_verifier, WebsiteService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let host = get_env_var("HOST");

    let jwks_url = get_env_var("JWKS_URL");
    let jwks_host = get_env_var("JWKS_HOST");

    let db_pool = init_db_pool(
        get_env_var("DB_HOST"),
        get_env_var("DB_PORT").parse().unwrap(),
        get_env_var("DB_USER"),
        get_env_var("DB_PASSWORD"),
        get_env_var("DB_DBNAME"),
        std::env::var("DB_ROOT_CERT").ok(),
    )?;
    migrate(&db_pool).await?;

    let cloudflare_service = CloudflareService::init(
        get_env_var("CLOUDFLARE_API_URL"),
        get_env_var("CLOUDFLARE_ZONE_ID"),
        get_env_var("MAIN_DOMAIN"),
        get_env_var("CLOUDFLARE_API_TOKEN"),
    );

    run_custom_hostnames_check(db_pool.clone(), cloudflare_service.clone())
        .await?;

    let (mut health_reporter, health_service) =
        tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<WebsiteServiceServer<WebsiteService>>()
        .await;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            tonic_health::pb::FILE_DESCRIPTOR_SET,
        )
        .register_encoded_file_descriptor_set(
            websites::api::sited_io::FILE_DESCRIPTOR_SET,
        )
        .build()
        .unwrap();

    let website_service = WebsiteService::build(
        db_pool,
        init_jwks_verifier(&jwks_host, &jwks_url)?,
        get_env_var("MAIN_DOMAIN"),
        ZitadelService::init(
            get_env_var("ZITADEL_API_URL"),
            get_env_var("ZITADEL_API_TOKEN"),
            get_env_var("ZITADEL_PROJECT_ID"),
        )
        .await?,
        cloudflare_service,
    );

    tracing::log::info!("gRPC+web server listening on {}", host);

    Server::builder()
        .accept_http1(true)
        .layer(
            TraceLayer::new_for_grpc()
                .on_request(LogOnRequest::default())
                .on_response(LogOnResponse::default())
                .on_failure(LogOnFailure::default()),
        )
        .layer(
            CorsLayer::new()
                .allow_headers([
                    AUTHORIZATION,
                    ACCEPT,
                    CONTENT_TYPE,
                    HeaderName::from_static("grpc-status"),
                    HeaderName::from_static("grpc-message"),
                    HeaderName::from_static("x-grpc-web"),
                    HeaderName::from_static("x-user-agent"),
                ])
                .allow_methods([Method::POST])
                .allow_origin(AllowOrigin::any())
                .allow_private_network(true),
        )
        .add_service(tonic_web::enable(reflection_service))
        .add_service(tonic_web::enable(health_service))
        .add_service(tonic_web::enable(website_service))
        .serve(host.parse().unwrap())
        .await?;

    Ok(())
}

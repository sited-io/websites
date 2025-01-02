use service_apis::sited_io::websites::v1::website_service_client::WebsiteServiceClient;
use tonic::transport::Channel;

async fn setup_websites_client(
    websites_url: String,
) -> WebsiteServiceClient<Channel> {
    tracing::info!("Running integration tests against: {websites_url}");
    WebsiteServiceClient::connect(websites_url).await.unwrap()
}

pub async fn setup(websites_url: String) -> WebsiteServiceClient<Channel> {
    setup_websites_client(websites_url).await
}

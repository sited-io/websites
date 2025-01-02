use service_apis::sited_io::websites::v1::ReplayWebsitesRequest;
use websites::get_env_var;

mod common;

#[tokio::test]
async fn test_replay() {
    let auth_url = get_env_var("TEST_AUTH_URL");
    let client_id = get_env_var("TEST_CLIENT_ID");
    let client_secret = get_env_var("TEST_CLIENT_SECRET");
    let websites_url = get_env_var("TEST_WEBSITES_URL");

    let mut auth_context =
        common::context::AuthContext::init(auth_url, client_id, client_secret)
            .await;
    let mut websites_client = common::setup(websites_url).await;

    let req = auth_context.auth_req(ReplayWebsitesRequest {}).await;
    let res = websites_client.replay_websites(req).await.unwrap();

    tracing::info!("RES: {:?}", res);
}

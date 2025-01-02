use http::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use tonic::Request;

pub struct AuthContext {
    auth_url: String,
    client_id: String,
    client_secret: String,
    user_id: String,
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
}

impl AuthContext {
    pub async fn init(
        auth_url: String,
        client_id: String,
        client_secret: String,
    ) -> Self {
        let mut auth_context = Self {
            auth_url,
            client_id,
            client_secret,
            user_id: String::new(),
            access_token: String::new(),
        };

        auth_context.refresh_access_token().await;
        tracing::info!("ACCESS_TOKEN: {}", auth_context.access_token);

        auth_context
    }

    pub fn user_id(&self) -> String {
        self.user_id.clone()
    }

    pub async fn auth_req<T>(&mut self, request: T) -> Request<T> {
        if self.access_token.is_empty() {
            self.refresh_access_token().await;
        }
        let mut request = Request::new(request);
        request.metadata_mut().insert(
            AUTHORIZATION.as_str(),
            format!("Bearer {}", self.access_token.clone())
                .parse()
                .unwrap(),
        );
        request
    }

    async fn refresh_access_token(&mut self) {
        let client = reqwest::Client::new();
        let form = [
            ("grant_type", "client_credentials"),
            ("scope", "openid profile urn:zitadel:iam:user:metadata"),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let res: AuthResponse = client
            .post(self.auth_url.clone())
            .header(CONTENT_TYPE, "application/x-www-form-urlencode")
            .form(&form)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        self.access_token = res.access_token;
    }
}

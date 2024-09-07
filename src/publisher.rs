use prost::Message;

use crate::api::sited_io::websites::v1::WebsiteResponse;

#[derive(Debug, Clone)]
pub struct Publisher {
    nats_client: async_nats::Client,
}

impl Publisher {
    const WEBSITE_UPSERT_SUBJECT: &'static str = "websites.website.upsert";
    const WEBSITE_DELETE_SUBJECT: &'static str = "websites.website.delete";

    pub fn new(nats_client: async_nats::Client) -> Self {
        Self { nats_client }
    }

    pub async fn publish_website(
        &self,
        website: &WebsiteResponse,
        is_delete: bool,
    ) {
        let subject = if is_delete {
            Self::WEBSITE_DELETE_SUBJECT
        } else {
            Self::WEBSITE_UPSERT_SUBJECT
        };
        if let Err(err) = self
            .nats_client
            .publish(subject, website.encode_to_vec().into())
            .await
        {
            tracing::log::error!("[WebsiteService.publish_website]: {}", err);
        }
    }
}

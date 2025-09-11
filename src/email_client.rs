use reqwest::Client;

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    pub sender: SubscriberEmail,
    pub http_client: Client,
    pub base_url: String,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            sender,
            base_url,
        }
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}

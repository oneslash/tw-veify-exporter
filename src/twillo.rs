use std::error::Error;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap},
};
use serde::Deserialize;
use thiserror::Error;

const BASE_URL: &str = "https://verify.twilio.com/v2/";
const API_ATTEMPTS_SUMMARY: &str = "Attempts/Summary";

#[derive(Debug, Error)]
pub enum Errors {
    #[error("API returned error {0}")]
    APIError(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttemptSummary {
    pub total_attempts: usize,
    pub total_converted: usize,
    pub total_unconverted: usize,
    pub conversion_rate_percentage: String,
}

#[derive(Debug, Clone)]
pub struct TwilloAPI {
    app_name: String,

    // Authorization params
    sid: String,
    token: String,
}

impl TwilloAPI {
    pub fn new(app_name: &str, sid: &str, token: &str) -> TwilloAPI {
        return TwilloAPI {
            app_name: app_name.to_owned(),
            sid: sid.to_owned(),
            token: token.to_owned(),
        };
    }

    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        let creds = format!("{}:{}", self.sid, self.token);
        let b64_creds = STANDARD.encode(creds.as_bytes());
        let auth = format!("Basic {}", b64_creds);
        headers.append(AUTHORIZATION, auth.parse().unwrap());

        return headers;
    }

    fn get_with_base(&self, path: &str) -> String {
        return format!("{}{}", BASE_URL, path);
    }
    
    pub async fn get_verification_summary(
        &self,
        after_date: &str,
        _country: Option<&str>,
    ) -> Result<AttemptSummary, Errors> {
        let api_url = self.get_with_base(API_ATTEMPTS_SUMMARY);
        let req = Client::new()
            .get(api_url)
            .headers(self.get_headers())
            .query(&[("DateCreatedAfter", after_date)])
            .send()
            .await;

        let resp = match req {
            Ok(r) => r,
            Err(e) => {
                let error_text = format!("Something happened {}", e.to_string());
                return Err(Errors::APIError(error_text.into()));
            }
        };

        if resp.status() != 200 {
            let error_text = format!("Something happened {}", resp.text().await.unwrap());
            return Err(Errors::APIError(error_text.into()));
        }

        let summary = resp.json::<AttemptSummary>().await.unwrap();

        Ok(summary)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_result() {
        let twillo = TwilloAPI::new("my-test", "", "");
        let result = twillo.get_verification_summary("2025-02-21T00:00:00Z", None);
        assert_eq!("{:?}", format!("{:?}", result.await.ok().unwrap()));
    }
}

use anyhow::Result;
use clap::Parser;
use reqwest::{Client, RequestBuilder, StatusCode, Url};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct AccountProperties {
    id: String,
    #[serde(rename = "mt4AccountID")]
    mt4_account_id: Option<u32>,
    tags: Vec<String>,
}

/// This is the json response recieved from /v3/accounts
#[derive(Deserialize, Debug)]
struct AccountsResponse {
    accounts: Vec<AccountProperties>,
}

pub struct V20Context {
    http: Client,
    rest_hostname: Url,
    streaming_hostname: Url,
    token: String,
}

impl V20Context {
    pub fn new_practice(token: impl Into<String>) -> Result<Self> {
        Ok(Self {
            http: Client::builder()
                .user_agent("oanda-v20-rs/0.1")
                .tcp_nodelay(true)
                .build()?,
            rest_hostname: "https://api-fxpractice.oanda.com".parse()?,
            streaming_hostname: "https://stream-fxpractice.oanda.com".parse()?,
            token: token.into(),
        })
    }

    #[cfg(test)]
    fn new_mock(rest_url: impl Into<String>, stream_url: impl Into<String>) -> Result<Self> {
        Ok(Self {
            http: Client::builder()
                .user_agent("oanda-v20-rs/0.1")
                .tcp_nodelay(true)
                .build()?,
            rest_hostname: Url::parse(&rest_url.into())?,
            streaming_hostname: Url::parse(&stream_url.into())?,
            token: "TEST_TOKEN".to_string(),
        })
    }

    fn auth(&self, req: RequestBuilder) -> RequestBuilder {
        req.bearer_auth(&self.token)
    }

    pub async fn accounts(&self) -> Result<Vec<AccountProperties>> {
        let url = self.rest_hostname.join("/v3/accounts")?;
        let res = self.auth(self.http.get(url)).send().await?;
        let bytes = res.error_for_status()?.bytes().await?;
        let acc: AccountsResponse = serde_json::from_slice(&bytes)?;
        Ok(acc.accounts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{header, method, path},
    };

    #[tokio::test]
    async fn create_practice_context() -> Result<()> {
        let svr = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("v3/accounts"))
            .and(header("Authorization", "Bearer TEST_TOKEN"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!(
                {
                    "accounts": [
                        {"id": "001-011-5838423-001", "tags": ["tag1", "tag2"]},
                        {"id": "001-011-5838423-002", "mt4AccountID": 42, "tags": []},
                    ]
                }
            )))
            .expect(1)
            .mount(&svr)
            .await;

        let ctx = V20Context::new_mock(svr.uri(), svr.uri())?;

        let accs = ctx.accounts().await?;
        assert_eq!(accs[0].id, "001-011-5838423-001");
        assert_eq!(accs[0].mt4_account_id, None);
        assert_eq!(accs[0].tags, vec!["tag1", "tag2"]);
        assert_eq!(accs[1].id, "001-011-5838423-002");
        assert_eq!(accs[1].mt4_account_id, Some(42));
        assert_eq!(accs[1].tags, Vec::<String>::new());

        Ok(())
    }
}

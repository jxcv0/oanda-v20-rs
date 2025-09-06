#![doc = include_str!("../README.md")]

pub use oanda_v20_openapi::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{header, method, path},
    };

    #[tokio::test]
    async fn create_practice_context() {
        let svr = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/accounts"))
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

        let config = apis::configuration::Configuration {
            base_path: svr.uri(),
            bearer_access_token: Some("TEST_TOKEN".to_string()),
            ..Default::default()
        };
        let response = apis::account_api::get_accounts(&config).await.unwrap();
        let Some(accs) = response.accounts else {
            panic!("No accounts");
        };

        assert_eq!(accs[0].id, Some("001-011-5838423-001".to_string()));
        assert_eq!(accs[0].mt4_account_id, None);
        assert_eq!(accs[0].tags, Some(vec!["tag1".to_string(), "tag2".to_string()]));
        assert_eq!(accs[1].id, Some("001-011-5838423-002".to_string()));
        assert_eq!(accs[1].mt4_account_id, Some(42));
        assert_eq!(accs[1].tags, Some(Vec::new()));
    }
}

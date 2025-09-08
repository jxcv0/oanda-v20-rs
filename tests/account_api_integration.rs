use oanda_v20_openapi::apis::configuration::Configuration;

#[tokio::test]
async fn main() {
    let token = std::env::var("API_TOKEN").expect("API_TOKEN environment variable not defined");
    let account = std::env::var("ACCOUNT_ID").expect("ACCOUNT_ID environment variable not defined");

    let config = Configuration {
        bearer_access_token: Some(token),
        ..Default::default()
    };

    let _ = oanda_v20_openapi::apis::account_api::get_account_instruments(&config, &account, None)
        .await
        .unwrap();
}

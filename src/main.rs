use clap::Parser;
use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AccountConfig {
    hostname: String,
    streaming_hostname: String,
    token: String,
    account: String,
}

#[derive(Parser, Debug)]
struct Args {
    /// Path to account config JSON file
    #[arg(short, long)]
    config: String,
}

#[derive(Deserialize, Debug)]
struct AccountProperties {
    id: String,
    #[serde(rename = "mt4AccountID")]
    mt4_account_id: Option<u32>,
    tags: Vec<String>
}

/// This is the json response recieved from /v3/accounts
#[derive(Deserialize, Debug)]
struct AccountsResponse {
    accounts: Vec<AccountProperties>
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // Load the account config
    let a = std::fs::read_to_string(args.config)?;
    let acc: AccountConfig = serde_json::from_str(&a)?;

    let baseurl = acc.hostname;
    let endpoint = Url::parse(&format!("https://{}/v3/accounts", baseurl))?;

    let client = reqwest::Client::new();
    let req = client
        .get(endpoint)
        .header("Content-Type", "application/json")
        .bearer_auth(acc.token)
        .build()?;
    let res = client.execute(req).await?;

    if res.status() == StatusCode::OK {
        let bytes = res.bytes().await?;
        let accres: AccountsResponse = serde_json::from_slice(&bytes)?;
        // Check that the account exists
        if accres.accounts.iter().any(|p| p.id == acc.account) {
            log::info!("Using account '{}'", acc.account);
        } else {
            anyhow::bail!("Did not find account '{}'", acc.account);
        }
    }
    Ok(())
}

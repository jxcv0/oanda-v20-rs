#![doc = include_str!("../README.md")]

use anyhow::Result;
use reqwest::{Client, RequestBuilder, StatusCode, Url};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Deserialize, Debug)]
pub struct AccountProperties {
    id: String,
    #[serde(rename = "mt4AccountID")]
    mt4_account_id: Option<u32>,
    tags: Vec<String>,
}

/// This is the json response recieved from /v3/accounts
#[derive(Deserialize, Debug)]
pub struct AccountsResponse {
    accounts: Vec<AccountProperties>,
}

/// This is the json response recieved from /v3/accounts/{accountID}
#[derive(Deserialize, Debug)]
pub struct SingleAccountResponse {
    account: Account,
    #[serde(rename = "lastTransactionID")]
    last_transaction_id: TransactionID
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
enum GuaranteedStopLossOrderMutability {
    Fixed,
    Replaceable,
    Cancelable,
    PriceWidenOnly,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GuaranteedStopLossOrderParameters {
    mutability_market_open: GuaranteedStopLossOrderMutability,
    mutability_market_halted: GuaranteedStopLossOrderMutability
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
enum GuaranteedStopLossOrderMode {
    Disabled,
    Allowed,
    Required,
}

type TransactionID = String;
type TradeID = String;

// TODO: Should these be fixed point numbers?
type DecimalNumber = String;
type AccountUnits = String;
type PriceValue = String;

#[derive(Deserialize, Debug)]
struct TradeSummary {
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Order {
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PositionSide {
    units : DecimalNumber,
    average_price : PriceValue,
    #[serde(rename = "tradeIDs")]
    trade_ids : Vec<TradeID>,
    pl : AccountUnits,
    #[serde(rename = "unrealizedPL")]
    unrealized_pl : AccountUnits,
    #[serde(rename = "resettablePL")]
    resettable_pl : AccountUnits,
    financing : AccountUnits,
    dividend_adjustment : AccountUnits,
    guaranteed_execution_fees : AccountUnits
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Position {
    instrument: String,
    pl: AccountUnits,
    #[serde(rename = "unrealizedPL")]
    unrealized_pl: AccountUnits,
    margin_used : AccountUnits,
    #[serde(rename = "resettablePL")]
    resettable_pl : AccountUnits,
    financing : AccountUnits,
    commission : AccountUnits,
    dividend_adjustment : AccountUnits,
    guaranteed_execution_fees : AccountUnits,
    long : PositionSide,
    short : PositionSide,
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Account {
    id: String,
    alias: Option<String>,
    currency: String,
    #[serde(rename = "createdByUserID")]
    created_by_user_id: u8,
    created_time: OffsetDateTime,
    guaranteed_stop_loss_order_parameters: GuaranteedStopLossOrderParameters,
    #[serde(rename = "resetablePLTime")]
    resetable_pl_time: OffsetDateTime,
    margin_rate: String, // TODO: This is a deciman number
    open_trade_count: i32,
    pending_order_count: i32,
    hedging_enabled: bool,
    #[serde(rename = "unrealizedPL")]
    unrealized_pl: String, // TODO: This is a decimal number
    #[serde(rename = "NAV")]
    nav: String, // TODO: This is a decimal number
    margin_used: String, // TODO: this is a decimal number
    margin_available: String, // TODO: this is a decimal number
    position_value: String, // TODO: this is a decimal number
    #[serde(rename = "marginCloseoutUnrealizedPL")]
    margin_closeout_unrealized_pl: String, // TODO: this is a decimal number
    margin_closeout_percent: String, // TODO: this is a decimal number
    margin_closeout_position_value: DecimalNumber,
    withdrawal_limit: AccountUnits,
    margin_call_marginused: AccountUnits,
    margin_call_percent: DecimalNumber,
    balance: AccountUnits,
    pl: AccountUnits,
    #[serde(rename = "resettablePL")]
    resettable_pl: AccountUnits,
    financing: AccountUnits,
    commission: AccountUnits,
    dividend_adjustment: AccountUnits,
    guaranteed_execution_fees: AccountUnits,
    margin_call_enter_time: OffsetDateTime,
    margin_call_extension_count : i32,
    last_margin_call_extension_time : OffsetDateTime,
    #[serde(rename = "lastTransactionID")]
    last_transaction_id: TransactionID,
    trades : Vec<TradeSummary>,
    positions : Vec<Position>,
    orders : Vec<Order>
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

    pub async fn accounts(&self) -> Result<AccountsResponse> {
        let url = self.rest_hostname.join("/v3/accounts")?;
        let res = self.auth(self.http.get(url)).send().await?;
        let bytes = res.error_for_status()?.bytes().await?;
        let acc: AccountsResponse = serde_json::from_slice(&bytes)?;
        Ok(acc)
    }

    pub async fn account(&self, account_id: &str) -> Result<SingleAccountResponse> {
        let url = self
            .rest_hostname
            .join(&format!("/v3/accounts/{}", account_id))?;
        let res = self.auth(self.http.get(url)).send().await?;
        let bytes = res.error_for_status()?.bytes().await?;
        let acc: SingleAccountResponse = serde_json::from_slice(&bytes)?;
        Ok(acc)
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
        assert_eq!(accs.accounts[0].id, "001-011-5838423-001");
        assert_eq!(accs.accounts[0].mt4_account_id, None);
        assert_eq!(accs.accounts[0].tags, vec!["tag1", "tag2"]);
        assert_eq!(accs.accounts[1].id, "001-011-5838423-002");
        assert_eq!(accs.accounts[1].mt4_account_id, Some(42));
        assert_eq!(accs.accounts[1].tags, Vec::<String>::new());

        Ok(())
    }
}

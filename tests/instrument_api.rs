use oanda_v20_openapi::models::{CandlesResponse, CandlestickData};
use serde_json::json;

/// This is to test a correction to the openapi.yaml spec where it specified floating point prices
/// would be given as f64s when in fact they are strings (presumably so that the api does not have
/// to specify the accuracy of the number)
#[test]
fn candlestick_data_prices_provided_as_string() {
    let json_body = json!({
        "instrument": "EUR_USD",
        "granularity": "S5",
        "candles": [
            {
                "complete": true,
                "volume":1,
                "time": "2025-09-05T20:58:40.000000000Z",
                "mid": {"o": "1.17186", "h": "1.17186", "l": "1.17186", "c": "1.17186"}
            },
            {
                "complete": true,
                "volume": 1,
                "time": "2025-09-05T20:58:45.000000000Z",
                "mid": {"o": "1.17185", "h": "1.17185", "l": "1.17185", "c": "1.17185"}
            }
        ]
    });

    let res: CandlesResponse = serde_json::from_value(json_body).unwrap();
    let candles = res.candles.unwrap();

    assert_eq!(candles.len(), 2);
    assert!(candles[0].complete.unwrap());
    assert_eq!(candles[0].volume, Some(1));
    assert_eq!(
        candles[0].time,
        Some("2025-09-05T20:58:40.000000000Z".to_string())
    );
    assert_eq!(
        candles[0].mid,
        Some(Box::new(CandlestickData {
            o: Some("1.17186".to_string()),
            h: Some("1.17186".to_string()),
            l: Some("1.17186".to_string()),
            c: Some("1.17186".to_string()),
        }))
    );
}

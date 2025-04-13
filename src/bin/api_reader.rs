//! A full program retrieving and parsing all tickers live.
//! Please find the benchmark and optimized parser under `benches`
#![feature(test)]

use measure_time::print_time;
use std::{fmt::Display, str::FromStr};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case, dead_code)]
struct Ticker {
    symbol: String,
    #[serde(deserialize_with = "parse_str")]
    priceChange: f32,
    #[serde(deserialize_with = "parse_str")]
    priceChangePercent: f32,
    #[serde(deserialize_with = "parse_str")]
    lastPrice: f32,
    #[serde(deserialize_with = "parse_str")]
    lastQty: f32,
    #[serde(deserialize_with = "parse_str")]
    open: f32,
    #[serde(deserialize_with = "parse_str")]
    high: f32,
    #[serde(deserialize_with = "parse_str")]
    low: f32,
    #[serde(deserialize_with = "parse_str")]
    volume: f32,
    #[serde(deserialize_with = "parse_str")]
    amount: f32,
    #[serde(deserialize_with = "parse_str")]
    bidPrice: f32,
    #[serde(deserialize_with = "parse_str")]
    askPrice: f32,
    #[serde(deserialize_with = "parse_str")]
    strikePrice: f32,
    #[serde(deserialize_with = "parse_str")]
    exercisePrice: f32,
    openTime: u64,
    closeTime: u64,
    firstTradeId: u64,
    tradeCount: u64,
}

fn parse_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: FromStr,
    T::Err: Display,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    s.parse::<T>().map_err(serde::de::Error::custom)
}

fn main() {
    const URL: &str = "https://eapi.binance.com/eapi/v1/ticker";
    let response = ureq::get(URL).call().expect("Error obtaining data");
    let json_reader = response.into_body().read_to_string().unwrap();

    let tickers: Vec<Ticker> = {
        print_time!("{:?}", "Parsing json");
        serde_json::from_str(&json_reader).expect("Error decoding")
    };

    println!("Received {} data points", tickers.len());
}

#[cfg(test)]
mod bench {
    #![allow(soft_unstable)]

    extern crate test;
    use super::*;
    use test::Bencher;

    const TICKER_EG: &'static str = r#"{
        "symbol":"ETH-250425-2100-C",
        "priceChange":"1.4",
        "priceChangePercent":"0.2693",
        "lastPrice":"6.6",
        "lastQty":"0.9",
        "open":"5.2",
        "high":"8.2",
        "low":"5.2",
        "volume":"33.83",
        "amount":"240.47",
        "bidPrice":"5.2",
        "askPrice":"5.6",
        "openTime":1744463031532,
        "closeTime":1744509383425,
        "firstTradeId":591,
        "tradeCount":8,
        "strikePrice":"2100",
        "exercisePrice":"1615.88071429"
    }"#;

    #[bench]
    fn bench_deser_ticker(b: &mut Bencher) {
        b.iter(|| {
            let tick: Ticker = serde_json::from_str(TICKER_EG).expect("Failure deserializing");
            assert_eq!(tick.priceChange, 1.4_f32)
        });
    }
}

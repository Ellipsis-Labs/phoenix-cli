use anyhow::anyhow;
use ellipsis_client::EllipsisClient;
use phoenix::program::MarketHeader;
use phoenix_sdk::sdk_client::SDKClient;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::{mem::size_of, str::FromStr};

use crate::helpers::{market_helpers::get_all_markets, print_helpers::print_market_summary_data};

pub async fn process_get_all_markets(client: &EllipsisClient) -> anyhow::Result<()> {
    let accounts = get_all_markets(client).await?;

    println!("Found {} market(s)", accounts.len());

    //Deserialize market accounts and print summary information
    for (market_pubkey, mut market_account) in accounts {
        let (header_bytes, _market_bytes) =
            market_account.data.split_at_mut(size_of::<MarketHeader>());

        let header = bytemuck::try_from_bytes(header_bytes)
            .map_err(|e| anyhow!("Error getting market header. Error: {:?}", e))?;

        print_market_summary_data(&market_pubkey, header);
    }
    Ok(())
}

pub async fn process_get_all_markets_no_gpa(
    client: &EllipsisClient,
    network_url: &str,
) -> anyhow::Result<()> {
    let markets = get_market_config().await?;

    println!("Found {} market(s)", markets.len());

    for market in markets {
        let market_pubkey = Pubkey::from_str(&market.market)?;
        let sdk = SDKClient::new(&market_pubkey, &client.payer, network_url).await;

        let market_account_data = sdk.client.get_account_data(&market_pubkey).await?;
        let (header_bytes, _market_bytes) = market_account_data.split_at(size_of::<MarketHeader>());
        let header: &MarketHeader = bytemuck::try_from_bytes(header_bytes)
            .map_err(|e| anyhow::anyhow!("Error getting market header. Error: {:?}", e))?;

        print_market_summary_data(&market_pubkey, header);
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct MarketStatic {
    market: String,
    base_ticker: String,
    quote_ticker: String,
    base_pubkey: String,
    quote_pubkey: String,
}

async fn get_market_config() -> anyhow::Result<Vec<MarketStatic>> {
    let body = reqwest::get(
        "https://raw.githubusercontent.com/Ellipsis-Labs/phoenix-sdk/master/mainnet_markets.json",
    )
    .await?
    .text()
    .await?;

    let markets: Vec<MarketStatic> = serde_json::from_str(&body)?;
    Ok(markets)
}

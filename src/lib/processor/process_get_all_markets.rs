use crate::helpers::{market_helpers::get_all_markets, print_helpers::print_market_summary_data};
use anyhow::anyhow;
use ellipsis_client::EllipsisClient;
use phoenix::program::MarketHeader;
use phoenix_sdk::sdk_client::SDKClient;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::{mem::size_of, str::FromStr};

pub async fn process_get_all_markets(client: &EllipsisClient) -> anyhow::Result<()> {
    let config = get_phoenix_config(client).await?;
    let accounts = get_all_markets(client).await?;

    println!("Found {} market(s)", accounts.len());

    //Deserialize market accounts and print summary information
    for (market_pubkey, mut market_account) in accounts {
        let (header_bytes, _market_bytes) =
            market_account.data.split_at_mut(size_of::<MarketHeader>());

        let header = bytemuck::try_from_bytes::<MarketHeader>(header_bytes)
            .map_err(|e| anyhow!("Error getting market header. Error: {:?}", e))?;

        let (base_mint_symbol, quote_mint_symbol) = {
            let base_mint = header.base_params.mint_key;
            let quote_mint = header.quote_params.mint_key;
            (
                config
                    .tokens
                    .iter()
                    .find(|t| t.mint == base_mint.to_string())
                    .map(|t| t.symbol.clone()),
                config
                    .tokens
                    .iter()
                    .find(|t| t.mint == quote_mint.to_string())
                    .map(|t| t.symbol.clone()),
            )
        };
        print_market_summary_data(&market_pubkey, header, base_mint_symbol, quote_mint_symbol);
    }
    Ok(())
}

pub async fn process_get_all_markets_no_gpa(
    client: &EllipsisClient,
    network_url: &str,
) -> anyhow::Result<()> {
    let config = get_phoenix_config(client).await?;
    let markets = config
        .markets
        .iter()
        .map(|m| m.market.clone())
        .collect::<Vec<String>>()
        .clone();

    println!("Found {} market(s)", markets.len());

    for market in markets {
        let market_pubkey = Pubkey::from_str(&market)?;
        let sdk = SDKClient::new(&client.payer, network_url).await?;

        let market_account_data = sdk.client.get_account_data(&market_pubkey).await?;
        let (header_bytes, _market_bytes) = market_account_data.split_at(size_of::<MarketHeader>());
        let header: &MarketHeader = bytemuck::try_from_bytes(header_bytes)
            .map_err(|e| anyhow::anyhow!("Error getting market header. Error: {:?}", e))?;

        let (base_mint_symbol, quote_mint_symbol) = {
            let base_mint = header.base_params.mint_key;
            let quote_mint = header.quote_params.mint_key;
            (
                config
                    .tokens
                    .iter()
                    .find(|t| t.mint == base_mint.to_string())
                    .map(|t| t.symbol.clone()),
                config
                    .tokens
                    .iter()
                    .find(|t| t.mint == quote_mint.to_string())
                    .map(|t| t.symbol.clone()),
            )
        };
        print_market_summary_data(&market_pubkey, header, base_mint_symbol, quote_mint_symbol);
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MasterConfig {
    pub tokens: Vec<TokenConfig>,
    pub markets: Vec<MarketConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenConfig {
    pub name: String,
    pub symbol: String,
    pub mint: String,
    pub logo_uri: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarketConfig {
    pub market: String,
    pub base_mint: String,
    pub quote_mint: String,
}

pub async fn get_phoenix_config(client: &EllipsisClient) -> anyhow::Result<MasterConfig> {
    let genesis = client.get_genesis_hash().await?;

    //hardcoded in the genesis hashes for mainnet and devnet
    let cluster = match genesis.to_string().as_str() {
        "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdpKuc147dw2N9d" => "mainnet-beta",
        "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG" => "devnet",
        _ => "localhost",
    };

    let body = reqwest::get(
        "https://raw.githubusercontent.com/Ellipsis-Labs/phoenix-sdk/master/master_config.json",
    )
    .await?
    .text()
    .await?;

    let config: HashMap<String, MasterConfig> = serde_json::from_str(&body)?;

    Ok(config
        .get(cluster)
        .ok_or_else(|| anyhow!("Failed to find market config"))?
        .clone())
}

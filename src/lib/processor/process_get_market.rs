use crate::helpers::print_helpers::*;
use phoenix::program::{load_with_dispatch, MarketHeader};
use phoenix_sdk::sdk_client::*;
use solana_sdk::pubkey::Pubkey;
use std::mem::size_of;

pub async fn process_get_market(market_pubkey: &Pubkey, sdk: &SDKClient) -> anyhow::Result<()> {
    let market_metadata = sdk.get_active_market_metadata();

    let market_account_data = sdk.client.get_account_data(market_pubkey).await?;
    let (header_bytes, market_bytes) = market_account_data.split_at(size_of::<MarketHeader>());
    let header: &MarketHeader = bytemuck::try_from_bytes(header_bytes)
        .map_err(|e| anyhow::anyhow!("Error getting market header. Error: {:?}", e))?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch(&header.market_size_params, market_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to load market. Error {:?}", e))?
        .inner;

    let taker_fees = market.get_taker_fee_bps();

    print_market_details(sdk, market_pubkey, market_metadata, &header, taker_fees).await
}

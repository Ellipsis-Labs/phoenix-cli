use crate::helpers::print_helpers::*;
use phoenix::program::{load_with_dispatch, MarketHeader};
use phoenix_sdk::sdk_client::*;
use solana_sdk::pubkey::Pubkey;
use std::mem::size_of;

pub async fn process_get_traders_for_market(
    market_pubkey: &Pubkey,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    // Get market account
    let mut market_account_data = sdk.client.get_account_data(market_pubkey).await?;
    let (header_bytes, market_bytes) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header: &MarketHeader = bytemuck::try_from_bytes(header_bytes)
        .map_err(|e| anyhow::anyhow!("Error getting market header. Error: {:?}", e))?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch(&header.market_size_params, market_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to load market. Error {:?}", e))?
        .inner;

    println!(
        "Found {} trader(s). Printing traders with locked or free lots",
        market.get_registered_traders().len()
    );

    // Print trader information
    for (pubkey, state) in market.get_registered_traders().iter() {
        print_trader_state(sdk, market_pubkey, pubkey, state)?;
    }

    Ok(())
}

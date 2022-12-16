use crate::helpers::print_helpers::*;
use borsh::BorshDeserialize;
use phoenix_sdk::sdk_client::*;
use phoenix_types::dispatch::load_with_dispatch_mut;
use phoenix_types::market::MarketHeader;
use solana_sdk::pubkey::Pubkey;
use std::mem::size_of;

pub async fn process_get_market(market_pubkey: &Pubkey, sdk: &SDKClient) -> anyhow::Result<()> {
    let market_metadata = sdk.get_active_market_metadata();

    let mut market_account_data = sdk.client.get_account_data(&market_pubkey).await?;
    let (header_bytes, market_bytes) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header = MarketHeader::try_from_slice(header_bytes)?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch_mut(&header.market_size_params, market_bytes)
        .unwrap()
        .inner;

    let taker_fees = market.get_taker_bps();

    print_market_details(&sdk, &market_pubkey, market_metadata, taker_fees).await;
    Ok(())
}
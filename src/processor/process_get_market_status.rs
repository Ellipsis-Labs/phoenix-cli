use borsh::BorshDeserialize;
use phoenix_types::market::MarketStatus;
use phoenix_sdk::sdk_client::*;
use phoenix_types::market::MarketHeader;
use solana_sdk::pubkey::Pubkey;
use std::mem::size_of;

pub async fn process_get_market_status(
    market_pubkey: &Pubkey,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    // Get market account
    let mut market_account_data = sdk.client.get_account_data(market_pubkey).await?;
    let (header_bytes, _) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header = MarketHeader::try_from_slice(header_bytes)?;

    let status = MarketStatus::from(header.status);
    println!("Market status: {}", status);
    Ok(())
}
use phoenix_sdk::sdk_client::*;
use solana_sdk::pubkey::Pubkey;

use crate::helpers::{market_helpers::get_book_levels, print_helpers::print_book};

pub async fn process_get_book_levels(
    market_pubkey: &Pubkey,
    sdk: &SDKClient,
    levels: u64,
) -> anyhow::Result<()> {
    let book = get_book_levels(market_pubkey, &sdk.client, levels).await?;
    if book.bids.is_empty() && book.asks.is_empty() {
        println!("Book is empty");
    } else {
        print_book(sdk, &book);
    }
    Ok(())
}

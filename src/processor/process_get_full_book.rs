use crate::helpers::market_helpers::*;
use crate::helpers::print_helpers::print_book;
use phoenix_sdk::sdk_client::*;
use solana_sdk::pubkey::Pubkey;

pub async fn process_get_full_book(
    market_pubkey: &Pubkey,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    let book = get_book_levels(&market_pubkey, &sdk.client, u64::MAX).await?;
    if book.bids.is_empty() && book.asks.is_empty() {
        println!("Book is empty");
    } else {
        print_book(&sdk, &book);
    } 
    Ok(())
}
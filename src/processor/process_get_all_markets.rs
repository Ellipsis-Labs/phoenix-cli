use crate::helpers::print_helpers::*;
use crate::helpers::market_helpers::*;
use borsh::BorshDeserialize;
use ellipsis_client::EllipsisClient;
use phoenix_types::market::MarketHeader;
use std::mem::size_of;

pub fn process_get_all_markets(client: &EllipsisClient) -> anyhow::Result<()> {
    let accounts = get_all_markets(&client)?;

    println!("Found {} market(s)", accounts.len());

    //Deserialize market accounts and print summary information
    for (market_pubkey, mut market_account) in accounts {
        let (header_bytes, _market_bytes) =
            market_account.data.split_at_mut(size_of::<MarketHeader>());

        let header = MarketHeader::try_from_slice(header_bytes)?;

        print_market_summary_data(&market_pubkey, &header);
    }
    Ok(())
}

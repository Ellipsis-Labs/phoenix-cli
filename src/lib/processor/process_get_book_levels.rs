use std::mem::size_of;

use phoenix::{
    program::{load_with_dispatch, MarketHeader},
    quantities::WrapperU64,
    state::{markets::RestingOrder, Side},
};
use phoenix_sdk::sdk_client::*;
use solana_sdk::{clock::Clock, commitment_config::CommitmentConfig, pubkey::Pubkey, sysvar};

use crate::helpers::print_helpers::{print_book_with_trader, LadderLevelEntry};

pub async fn process_get_book_levels(
    market_pubkey: &Pubkey,
    sdk: &SDKClient,
    levels: u64,
) -> anyhow::Result<()> {
    let mut ask_entries: Vec<LadderLevelEntry> = Vec::with_capacity(levels as usize);
    let mut bid_entries: Vec<LadderLevelEntry> = Vec::with_capacity(levels as usize);

    // let meta = sdk.get_market_metadata(market_pubkey).await?;
    // Get market account
    let mut market_and_clock = sdk
        .client
        .get_multiple_accounts_with_commitment(
            &[*market_pubkey, sysvar::clock::id()],
            CommitmentConfig::confirmed(),
        )
        .await?
        .value;

    let market_account_data = market_and_clock
        .remove(0)
        .ok_or_else(|| anyhow::Error::msg("Market account not found"))?
        .data;

    let clock_account_data = market_and_clock
        .remove(0)
        .ok_or_else(|| anyhow::Error::msg("Clock account not found"))?
        .data;

    let clock: Clock = bincode::deserialize(&clock_account_data)
        .map_err(|_| anyhow::Error::msg("Error deserializing clock"))?;

    let (header_bytes, market_bytes) = market_account_data.split_at(size_of::<MarketHeader>());
    let header: &MarketHeader = bytemuck::try_from_bytes(header_bytes)
        .map_err(|e| anyhow::anyhow!("Error getting market header. Error: {:?}", e))?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch(&header.market_size_params, market_bytes)?.inner;

    // If not present, use u32::MAX instead of aborting.
    // This will simply not print any markers.
    let trader_index = market.get_trader_index(&sdk.trader).unwrap_or(u32::MAX);
    let book_bids = market.get_book(Side::Bid);
    let book_asks = market.get_book(Side::Ask);

    println!("Open Bids");
    let mut open_bids = vec![];
    open_bids.push(format!(
        "{0: <20} | {1: <20} | {2: <10} | {3: <10} | {4: <15} | {5: <15} ",
        "ID", "Price (ticks)", "Price", "Quantity", "Slots Remaining", "Seconds Remaining"
    ));

    for (order_id, order) in book_bids.iter() {
        // Check if order is expired
        if order.is_expired(clock.slot, clock.unix_timestamp as u64) {
            continue;
        }

        // Check if entry is present
        if let Some(ref mut entry) = bid_entries
            .iter_mut()
            .find(|entry| entry.tick == order_id.price_in_ticks)
        {
            // If entry is present, add to amount
            entry.lots += order.num_base_lots.as_u64();

            // Flag trader if present
            entry.trader_present |= order.trader_index == trader_index as u64;
        }

        // Otherwise, check length before attempting to add entry
        if bid_entries.len() < levels as usize {
            bid_entries.push(LadderLevelEntry {
                tick: order_id.price_in_ticks.as_u64(),
                lots: order.num_base_lots.as_u64(),
                trader_present: order.trader_index == trader_index as u64,
            })
        } else {
            break;
        }
    }

    for (order_id, order) in book_asks.iter() {
        // Check if order is expired
        if order.is_expired(clock.slot, clock.unix_timestamp as u64) {
            continue;
        }

        // Check if entry is present
        if let Some(ref mut entry) = ask_entries
            .iter_mut()
            .find(|entry| entry.tick == order_id.price_in_ticks)
        {
            // If entry is present, add to amount
            entry.lots += order.num_base_lots.as_u64();

            // Flag trader if present
            entry.trader_present |= order.trader_index == trader_index as u64;
        }

        // Otherwise, check length before attempting to add entry
        if ask_entries.len() < levels as usize {
            ask_entries.push(LadderLevelEntry {
                tick: order_id.price_in_ticks.as_u64(),
                lots: order.num_base_lots.as_u64(),
                trader_present: order.trader_index == trader_index as u64,
            })
        } else {
            break;
        }
    }

    print_book_with_trader(sdk, market_pubkey, &bid_entries, &ask_entries)?;

    Ok(())
}

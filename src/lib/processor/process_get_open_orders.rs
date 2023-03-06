use phoenix::program::{load_with_dispatch, MarketHeader};
use phoenix::quantities::WrapperU64;
use phoenix::state::Side;
use phoenix_sdk::sdk_client::*;
use solana_sdk::pubkey::Pubkey;
use std::mem::size_of;

use crate::helpers::print_helpers::get_precision;

pub async fn process_get_open_orders(
    market_pubkey: &Pubkey,
    trader_pubkey: &Pubkey,
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

    let trader_index = market
        .get_trader_index(trader_pubkey)
        .ok_or_else(|| anyhow::anyhow!("Trader not found"))?;
    println!("Trader Index: {}", trader_index);
    let book_bids = market.get_book(Side::Bid);
    let book_asks = market.get_book(Side::Ask);
    let price_precision: usize =
        get_precision(10_u64.pow(sdk.quote_decimals) / sdk.tick_size_in_quote_atoms_per_base_unit);
    let size_precision: usize = get_precision(sdk.num_base_lots_per_base_unit);

    println!("Open Bids");
    let mut open_bids = vec![];
    open_bids.push(format!(
        "{0: <20} | {1: <20} | {2: <10} | {3: <10}",
        "ID", "Price (ticks)", "Price", "Quantity"
    ));
    for (order_id, order) in book_bids.iter() {
        if order.trader_index as u32 == trader_index {
            open_bids.push(format!(
                "{0: <20} | {1: <20} | {2: <10} | {3: <10}",
                order_id.order_sequence_number as i64,
                order_id.price_in_ticks,
                format!(
                    "{:.1$}",
                    sdk.ticks_to_float_price(order_id.price_in_ticks.into()),
                    price_precision
                ),
                format!(
                    "{:.1$}",
                    order.num_base_lots.as_u64() as f64 * sdk.base_lots_to_base_units_multiplier(),
                    size_precision
                ),
            ));
        }
    }
    open_bids.iter().for_each(|line| println!("{}", line));

    println!();
    println!("Open Asks");
    let mut open_asks = vec![];
    open_asks.push(format!(
        "{0: <20} | {1: <20} | {2: <10} | {3: <10}",
        "ID", "Price (ticks)", "Price", "Quantity",
    ));
    for (order_id, order) in book_asks.iter() {
        if order.trader_index as u32 == trader_index {
            open_asks.push(format!(
                "{0: <20} | {1: <20} | {2: <10} | {3: <10}",
                order_id.order_sequence_number as i64,
                order_id.price_in_ticks,
                format!(
                    "{:.1$}",
                    sdk.ticks_to_float_price(order_id.price_in_ticks.as_u64()),
                    price_precision
                ),
                format!(
                    "{:.1$}",
                    order.num_base_lots.as_u64() as f64 * sdk.base_lots_to_base_units_multiplier(),
                    size_precision,
                ),
            ));
        }
    }
    open_asks.iter().for_each(|line| println!("{}", line));

    Ok(())
}

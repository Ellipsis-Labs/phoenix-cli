use borsh::BorshDeserialize;
use phoenix_sdk::sdk_client::*;
use phoenix_types::dispatch::load_with_dispatch_mut;
use phoenix_types::enums::Side;
use phoenix_types::market::MarketHeader;
use solana_sdk::pubkey::Pubkey;
use std::mem::size_of;

pub async fn process_get_open_orders(
    market_pubkey: &Pubkey,
    trader_pubkey: &Pubkey,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    // Get market account
    let mut market_account_data = sdk.client.get_account_data(market_pubkey).await?;
    let (header_bytes, market_bytes) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header = MarketHeader::try_from_slice(header_bytes)?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch_mut(&header.market_size_params, market_bytes)
        .unwrap()
        .inner;

    let trader_index = market
        .get_trader_index(trader_pubkey)
        .ok_or_else(|| anyhow::anyhow!("Trader not found"))?;

    let book_bids = market.get_book(Side::Bid);
    let book_asks = market.get_book(Side::Ask);

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
                order_id.order_sequence_number,
                order_id.price_in_ticks,
                sdk.ticks_to_float_price(order_id.price_in_ticks),
                order.num_base_lots as f64 * sdk.base_lots_to_base_units_multiplier()
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
                order_id.order_sequence_number,
                order_id.price_in_ticks,
                sdk.ticks_to_float_price(order_id.price_in_ticks),
                order.num_base_lots as f64 * sdk.base_lots_to_base_units_multiplier()
            ));
        }
    }
    open_asks.iter().for_each(|line| println!("{}", line));

    Ok(())
}

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
    let mut market_account_data = sdk.client.get_account_data(&market_pubkey).await?;
    let (header_bytes, market_bytes) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header = MarketHeader::try_from_slice(header_bytes)?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch_mut(&header.market_size_params, market_bytes)
        .unwrap()
        .inner;

    let trader_index = market.get_trader_index(&trader_pubkey).ok_or(anyhow::anyhow!("Trader not found"))?;

    let book_bids = market.get_book(Side::Bid);
    let book_asks = market.get_book(Side::Ask);

    let mut open_bids = vec![];
    for (order_id, order) in book_bids.iter() {
        if order.trader_index as u32 == trader_index {
            open_bids.push(
                format!(
                    "Price: {}, Size: {:.3}",
                    sdk.core.ticks_to_float_price(order_id.price_in_ticks),
                    sdk.base_lots_to_base_units_multiplier() * order.num_base_lots as f64
                )
            );
        }
    }
    println!("Open Bids: {:?}", open_bids);

    let mut open_asks = vec![];
    for (order_id, order) in book_asks.iter() {
        if order.trader_index as u32 == trader_index {
            open_asks.push(
                format!(
                    "Price: {}, Size: {:.3}",
                    sdk.core.ticks_to_float_price(order_id.price_in_ticks),
                    sdk.base_lots_to_base_units_multiplier() * order.num_base_lots as f64
                )
            );
        }
    }
    println!("Open Asks: {:?}", open_asks);

    Ok(())
}

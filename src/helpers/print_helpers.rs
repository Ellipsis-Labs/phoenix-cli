use phoenix_sdk::sdk_client::*;
use phoenix_types::enums::Side;
use phoenix_types::instructions::get_vault_address;
use phoenix_types::market::{MarketHeader, TraderState, Ladder};
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;

pub fn print_book(sdk: &SDKClient, book: &Ladder) { 
    let bids = book.bids.iter().map(|b| {
        format!(
            "Price: {}, Size: {:.3}",
            sdk.core.ticks_to_float_price(b.price_in_ticks),
            sdk.base_lots_to_base_units_multiplier() * b.size_in_base_lots as f64
        )
    }).collect::<Vec<String>>();
    println!("Bids: {:?}", bids);

    let asks = book.asks.iter().map(|a| {
        format!(
            "Price: {}, Size: {:.3}",
            sdk.core.ticks_to_float_price(a.price_in_ticks),
            sdk.base_lots_to_base_units_multiplier() * a.size_in_base_lots as f64
        )
    }).collect::<Vec<String>>();
    println!("Asks: {:?}", asks);


}
pub fn print_market_summary_data(market_pubkey: &Pubkey, header: &MarketHeader) {
    let base_pubkey = header.base_params.mint_key;
    let quote_pubkey = header.quote_params.mint_key;

    println!("--------------------------------------------");
    println!("Market: {:?}", market_pubkey);
    println!("Base Token: {:?}", base_pubkey);
    println!("Quote Token: {:?}", quote_pubkey);
    println!("Authority: {:?}", header.authority);
}

pub async fn print_market_details(
    sdk: &SDKClient,
    market_pubkey: &Pubkey,
    market_metadata: &MarketMetadata,
    taker_fees: u16,
) {
    let base_pubkey = market_metadata.base_mint;
    let quote_pubkey = market_metadata.quote_mint;

    let base_vault = get_vault_address(market_pubkey, &base_pubkey).0;
    let quote_vault = get_vault_address(market_pubkey, &quote_pubkey).0;

    let base_vault_acct =
        spl_token::state::Account::unpack(&sdk.client.get_account(&base_vault).await.unwrap().data)
            .unwrap();

    let quote_vault_acct = spl_token::state::Account::unpack(
        &sdk.client.get_account(&quote_vault).await.unwrap().data,
    )
    .unwrap();

    println!(
        "Base Vault balance: {:.3}",
        get_decimal_string(base_vault_acct.amount, sdk.base_decimals).parse::<f64>().unwrap()
    );

    println!(
        "Quote Vault balance: {:.3}",
        get_decimal_string(quote_vault_acct.amount, sdk.quote_decimals).parse::<f64>().unwrap()
    );

    println!("Base Token: {}", base_pubkey);
    println!("Quote Token: {}", quote_pubkey);
    println!(
        "Base Lot Size: {}",
        get_decimal_string(market_metadata.base_lot_size, market_metadata.base_decimals),
    );

    println!(
        "Quote Lot Size: {}",
        get_decimal_string(
            market_metadata.quote_lot_size,
            market_metadata.quote_decimals
        )
    );
    println!(
        "Tick size (quote atoms per base unit): {}",
        market_metadata.tick_size_in_quote_atoms_per_base_unit
    );
    println!("Taker fees in bips: {}", taker_fees);
}

pub fn print_trader_state(sdk: &SDKClient, pubkey: &Pubkey, state: &TraderState) {
    if state.base_lots_locked == 0
        && state.base_lots_free == 0
        && state.quote_lots_locked == 0
        && state.quote_lots_free == 0
    {
        return;
    }
    println!("--------------------------------");
    println!("Trader pubkey: {:?}", pubkey);
    println!(
        "Base token locked: {}",
        get_decimal_string(
            sdk.base_lots_to_base_amount(state.base_lots_locked),
            sdk.base_decimals
        )
    );
    println!(
        "Base token free: {}",
        get_decimal_string(
            sdk.base_lots_to_base_amount(state.base_lots_free),
            sdk.base_decimals
        )
    );
    println!(
        "Quote token locked: {}",
        get_decimal_string(
            sdk.quote_lots_to_quote_amount(state.quote_lots_locked),
            sdk.quote_decimals
        )
    );
    println!(
        "Quote token free: {}",
        get_decimal_string(
            sdk.quote_lots_to_quote_amount(state.quote_lots_free),
            sdk.quote_decimals
        )
    );
}

pub fn log_market_events(sdk: &SDKClient, market_events: Vec<PhoenixEvent>) {
    for event in market_events {
        match event.details {
            MarketEventDetails::Fill(fill) => {
                if event.market != sdk.active_market_key {
                    continue;
                }
                let Fill {
                    maker,
                    taker,
                    price_in_ticks,
                    base_lots_filled,
                    side_filled,
                    ..
                } = fill;
                let keys = initialize_log(&event, "Fill".to_string());
                let fill_data = vec![
                    maker.to_string(),
                    taker.to_string(),
                    (sdk.ticks_to_float_price(price_in_ticks)).to_string(),
                    format!("{:?}", side_filled),
                    get_decimal_string(
                        sdk.base_lots_to_base_amount(base_lots_filled),
                        sdk.base_decimals,
                    ),
                ];
                println!("{}", finalize_log(keys, fill_data));
            }
            MarketEventDetails::Place(place) => {
                if event.market != sdk.active_market_key {
                    continue;
                }
                let Place {
                    order_sequence_number,
                    client_order_id: _,
                    maker,
                    price_in_ticks,
                    base_lots_placed,
                } = place;
                let side = Side::from_order_sequence_number(order_sequence_number);
                let keys = initialize_log(&event, "Place".to_string());
                let place_data = vec![
                    maker.to_string(),
                    "".to_string(),
                    (sdk.ticks_to_float_price(price_in_ticks)).to_string(),
                    format!("{:?}", side),
                    get_decimal_string(
                        sdk.base_lots_to_base_amount(base_lots_placed),
                        sdk.base_decimals,
                    ),
                ];

                println!("{}", finalize_log(keys, place_data));
            }
            MarketEventDetails::Reduce(reduce) => {
                if event.market != sdk.active_market_key {
                    continue;
                }
                let Reduce {
                    order_sequence_number,
                    maker,
                    price_in_ticks,
                    base_lots_removed,
                    ..
                } = reduce;
                let side = Side::from_order_sequence_number(order_sequence_number);
                let keys = initialize_log(&event, "Reduce".to_string());

                let reduce_data = vec![
                    maker.to_string(),
                    "".to_string(),
                    (sdk.ticks_to_float_price(price_in_ticks)).to_string(),
                    format!("{:?}", side),
                    get_decimal_string(
                        sdk.base_lots_to_base_amount(base_lots_removed),
                        sdk.base_decimals,
                    ),
                ];
                println!("{}", finalize_log(keys, reduce_data));
            }
            MarketEventDetails::FillSummary(fill_summary) => {
                let FillSummary {
                    total_quote_fees,
                    ..
                } = fill_summary;
                println!("Total quote token fees paid: {}", sdk.quote_amount_to_quote_unit_as_float(total_quote_fees));
            }
            _ => {
                continue;
            }
        }
    }
}
pub fn initialize_log(event: &PhoenixEvent, event_type: String) -> Vec<String> {
    let base_schema: Vec<String> = vec![
        "market".to_string(),
        "event_type".to_string(),
        "timestamp".to_string(),
        "signature".to_string(),
        "slot".to_string(),
        "sequence_number".to_string(),
        "event_index".to_string(),
    ];
    let base = vec![
        event.market.to_string(),
        event_type,
        event.timestamp.to_string(),
        event.signature.to_string(),
        event.slot.to_string(),
        event.sequence_number.to_string(),
        event.event_index.to_string(),
    ];
    base_schema
        .iter()
        .zip(base.iter())
        .map(|(a, b)| format!("{}: {}", a, b))
        .collect::<Vec<String>>()
}

pub fn finalize_log(mut log: Vec<String>, data: Vec<String>) -> String {
    let event_schema: Vec<String> = vec![
        "maker".to_string(),
        "taker".to_string(),
        "price".to_string(),
        "side".to_string(),
        "quantity".to_string(),
    ];
    log.extend_from_slice(
        &event_schema
            .iter()
            .zip(data.iter())
            .map(|(a, b)| format!("{}: {}", a, b))
            .collect::<Vec<String>>(),
    );
    log.join(", ")
}

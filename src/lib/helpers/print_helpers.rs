use std::collections::BTreeMap;
use std::mem::size_of;

use colored::Colorize;
use phoenix::program::status::MarketStatus;
use phoenix::program::MarketHeader;
use phoenix::program::{get_vault_address, load_with_dispatch};
use phoenix::quantities::WrapperU64;
use phoenix::state::{markets::Ladder, Side, TraderState};
use phoenix_sdk::sdk_client::*;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;

pub fn print_book(sdk: &SDKClient, book: &Ladder) {
    let asks = book.asks.iter().map(|lvl| {
        (
            sdk.ticks_to_float_price(lvl.price_in_ticks),
            lvl.size_in_base_lots as f64 * sdk.base_lots_to_base_units_multiplier(),
        )
    });

    let bids = book.bids.iter().map(|lvl| {
        (
            sdk.ticks_to_float_price(lvl.price_in_ticks),
            lvl.size_in_base_lots as f64 * sdk.base_lots_to_base_units_multiplier(),
        )
    });
    let price_precision: usize =
        get_precision(10_u64.pow(sdk.quote_decimals) / sdk.tick_size_in_quote_atoms_per_base_unit);
    let size_precision: usize = get_precision(sdk.num_base_lots_per_base_unit);
    let bid_strings = bids
        .into_iter()
        .map(|(price, size)| {
            let p = format!("{:.1$}", price, price_precision);
            let s = format!("{:.1$}", size, size_precision).green();
            (s, p)
        })
        .collect::<Vec<_>>();

    let bid_width = bid_strings.iter().map(|(s, _)| s.len()).max().unwrap_or(0) + 1;

    let ask_strings = asks
        .into_iter()
        .rev()
        .map(|(price, size)| {
            let p = format!("{:.1$}", price, price_precision);
            let s = format!("{:.1$}", size, size_precision).red();
            (p, s)
        })
        .collect::<Vec<_>>();

    let price_width = bid_strings
        .iter()
        .zip(ask_strings.iter())
        .map(|(a, b)| a.0.len().max(b.1.len()))
        .max()
        .unwrap_or(0);

    let ask_width = ask_strings.iter().map(|(_, s)| s.len()).max().unwrap_or(0) + 1;

    for (price, size) in ask_strings {
        let str = format!(
            "{:bid_width$} {:>price_width$} {:>ask_width$}",
            "", price, size
        );
        println!("{}", str);
    }
    for (size, price) in bid_strings {
        let str = format!(
            "{:>bid_width$} {:>price_width$} {:ask_width$}",
            size, price, ""
        );
        println!("{}", str);
    }
}

pub fn get_precision(mut target: u64) -> usize {
    let mut prime_factors = BTreeMap::new();
    let mut candidate = 2;
    while target > 1 {
        if target % candidate == 0 {
            *prime_factors.entry(candidate).or_insert(0) += 1;
            target /= candidate;
        } else {
            candidate += 1;
        }
    }
    let precision =
        (*prime_factors.get(&2).unwrap_or(&0)).max(*prime_factors.get(&5).unwrap_or(&0));
    if precision == 0 {
        // In the off chance that the target does not have 2 or 5 as a prime factor,
        // we'll just return a precision of 3 decimals.
        3
    } else {
        precision
    }
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
    market_header: &MarketHeader,
    taker_fees: u64,
) -> anyhow::Result<()> {
    let base_pubkey = market_metadata.base_mint;
    let quote_pubkey = market_metadata.quote_mint;

    let base_vault = get_vault_address(market_pubkey, &base_pubkey).0;
    let quote_vault = get_vault_address(market_pubkey, &quote_pubkey).0;

    let base_vault_acct =
        spl_token::state::Account::unpack(&sdk.client.get_account(&base_vault).await?.data)?;

    let quote_vault_acct =
        spl_token::state::Account::unpack(&sdk.client.get_account(&quote_vault).await?.data)?;

    // Get market account
    let mut market_account_data = sdk.client.get_account_data(market_pubkey).await?;
    let (header_bytes, market_bytes) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header: &MarketHeader = bytemuck::try_from_bytes(header_bytes)
        .map_err(|e| anyhow::anyhow!("Error getting market header. Error: {:?}", e))?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch(&header.market_size_params, market_bytes)?.inner;

    println!("--------------------------------------------");
    println!("Market: {}", market_pubkey);
    println!("Status: {}", MarketStatus::from(market_header.status));
    println!("Authority: {}", market_header.authority);
    println!("Sequence number: {}", market_header.market_sequence_number);

    println!(
        "Base Vault balance: {:.3}",
        get_decimal_string(base_vault_acct.amount, sdk.base_decimals).parse::<f64>()?
    );

    println!(
        "Quote Vault balance: {:.3}",
        get_decimal_string(quote_vault_acct.amount, sdk.quote_decimals).parse::<f64>()?
    );

    println!("Base Token: {}", base_pubkey);
    println!("Quote Token: {}", quote_pubkey);

    println!("Base vault key: {}", market_header.base_params.vault_key);
    println!("Quote vault key: {}", market_header.quote_params.vault_key);

    println!(
        "Base Lot Size, in whole units: {}",
        get_decimal_string(market_metadata.base_lot_size, market_metadata.base_decimals),
    );
    println!(
        "Quote Lot Size, in whole units: {}",
        get_decimal_string(
            market_metadata.quote_lot_size,
            market_metadata.quote_decimals
        )
    );
    println!(
        "Tick size in quote atoms per base unit: {}",
        get_decimal_string(
            market_metadata.tick_size_in_quote_atoms_per_base_unit,
            market_metadata.quote_decimals
        )
    );
    println!("Taker fees in basis points: {}", taker_fees);
    println!("Fee destination pubkey: {:?}", market_header.fee_recipient);
    println!(
        "Raw base units per base unit: {}",
        market_metadata.raw_base_units_per_base_unit
    );
    println!("Market Size Params: {:?}", market_header.market_size_params);
    println!("Market authority pubkey: {:?}", market_header.authority);
    println!("Successor pubkey: {:?}", market_header.successor);

    println!(
        "Uncollected fees, in quote units: {}",
        get_decimal_string(
            sdk.quote_lots_to_quote_atoms(market.get_uncollected_fee_amount().as_u64()),
            market_metadata.quote_decimals
        )
    );
    println!(
        "Collected fees, in quote units: {}",
        get_decimal_string(
            sdk.quote_lots_to_quote_atoms(market.get_collected_fee_amount().as_u64()),
            market_metadata.quote_decimals
        )
    );

    Ok(())
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
            sdk.base_lots_to_base_atoms(state.base_lots_locked.into()),
            sdk.base_decimals
        )
    );
    println!(
        "Base token free: {}",
        get_decimal_string(
            sdk.base_lots_to_base_atoms(state.base_lots_free.into()),
            sdk.base_decimals
        )
    );
    println!(
        "Quote token locked: {}",
        get_decimal_string(
            sdk.quote_lots_to_quote_atoms(state.quote_lots_locked.into()),
            sdk.quote_decimals
        )
    );
    println!(
        "Quote token free: {}",
        get_decimal_string(
            sdk.quote_lots_to_quote_atoms(state.quote_lots_free.into()),
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
                        sdk.base_lots_to_base_atoms(base_lots_filled),
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
                        sdk.base_lots_to_base_atoms(base_lots_placed),
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
                        sdk.base_lots_to_base_atoms(base_lots_removed),
                        sdk.base_decimals,
                    ),
                ];
                println!("{}", finalize_log(keys, reduce_data));
            }
            MarketEventDetails::FillSummary(fill_summary) => {
                let FillSummary {
                    total_quote_fees, ..
                } = fill_summary;
                println!(
                    "Total quote token fees paid: {}",
                    sdk.quote_atoms_to_quote_unit_as_float(total_quote_fees)
                );
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

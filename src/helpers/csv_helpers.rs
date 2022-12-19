use phoenix_sdk::sdk_client::*;
use phoenix_types::enums::Side;
use csv::Writer;

pub fn market_events_to_csv(sdk: &SDKClient, market_events: Vec<PhoenixEvent>, file_path: String) -> anyhow::Result<()> {
    let mut wtr = Writer::from_path(file_path).expect("Failed to create csv file");

    wtr.write_record(&[
        "market",
        "event_type",
        "timestamp",
        "signature",
        "slot",
        "sequence_number",
        "event_index",
        "maker",
        "taker",
        "price",
        "side",
        "quantity",
    ])
    .expect("Failed to write to csv");

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
                let mut log = initialize_csv_log(&event, "Fill".to_string());
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
                log.extend_from_slice(&fill_data);
                wtr.write_record(&log)?;
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
                let mut log = initialize_csv_log(&event, "Place".to_string());
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
                log.extend_from_slice(&place_data);
                wtr.write_record(&log)?;
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
                let mut log = initialize_csv_log(&event, "Reduce".to_string());

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
                log.extend_from_slice(&reduce_data);
                wtr.write_record(&log)?;
            }
            _ => {
                continue;
            }
        }
    }
    wtr.flush()?;
    Ok(())
}

pub fn initialize_csv_log(event: &PhoenixEvent, event_type: String) -> Vec<String> {
    vec![
        event.market.to_string(),
        event_type,
        event.timestamp.to_string(),
        event.signature.to_string(),
        event.slot.to_string(),
        event.sequence_number.to_string(),
        event.event_index.to_string(),
    ]

}


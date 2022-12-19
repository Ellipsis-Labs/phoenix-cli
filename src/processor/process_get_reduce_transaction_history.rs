use crate::helpers::market_helpers::*;
use crate::helpers::print_helpers::*;
use phoenix_sdk::sdk_client::*;
use solana_sdk::pubkey::Pubkey;
use crate::helpers::csv_helpers::market_events_to_csv;

pub async fn process_get_reduce_transaction_history(
    market_pubkey: &Pubkey,
    trader_pubkey: &Pubkey,
    lookback_slots: u64,
    save_csv: bool,
    file_path: String,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    let transaction_history = get_transaction_history(market_pubkey, trader_pubkey, lookback_slots, &sdk).await?;
    let mut events = vec![];
    let mut failures = vec![];
    for sig in transaction_history {
        let transaction_events = sdk
        .parse_events_from_transaction(&sig)
        .await; 

        match transaction_events {
            Some(transaction_events) => events.extend(transaction_events),
            None => failures.push(sig),
        }
    }

    let reduce_events = events.into_iter().filter(|x| match x.details {
        MarketEventDetails::Reduce(_) => true,
        _ => false,
    }).collect::<Vec<PhoenixEvent>>();

    if reduce_events.is_empty() {
        println!(
            "No reduces found for {} in the last {} slots",
            trader_pubkey, lookback_slots
        );
    }

    if save_csv {
        market_events_to_csv(sdk, reduce_events, file_path)?;
    } else {
        log_market_events(sdk, reduce_events);
    }
    
    if !failures.is_empty() {
        println!("Failed to parse {} signature(s):" , failures.len());
        for sig in failures {
            println!("{}", sig);
        }
    }


    Ok(())
}
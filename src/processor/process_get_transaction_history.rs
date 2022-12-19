use crate::helpers::market_helpers::*;
use crate::helpers::print_helpers::*;
use phoenix_sdk::sdk_client::*;
use solana_sdk::pubkey::Pubkey;
use crate::helpers::csv_helpers::market_events_to_csv;

pub async fn process_get_transaction_history(
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
            None => {
                // parse (seemingly) arbitrary fails a low % of the time, so we'll retry once
                let retry = sdk.parse_events_from_transaction(&sig).await;
                match retry {
                    Some(retry) => events.extend(retry),
                    None => failures.push(sig),
                }
            },
        }
    }

    if save_csv {
        market_events_to_csv(sdk, events, file_path)?;
    } else {
        log_market_events(sdk, events);
    }
    
    if !failures.is_empty() {
        println!("Failed to parse {} signature(s):" , failures.len());
        for sig in failures {
            println!("{}", sig);
        }
    }


    Ok(())
}
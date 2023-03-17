use crate::helpers::print_helpers::*;
use phoenix_sdk::sdk_client::*;
use solana_sdk::signature::Signature;

pub async fn process_get_transaction(
    signature: &Signature,
    sdk: &mut SDKClient,
) -> anyhow::Result<()> {
    let events = sdk
        .parse_events_from_transaction(signature)
        .await
        .ok_or_else(|| anyhow::anyhow!("Failed to parse events from transaction"))?;
    log_market_events(sdk, events).await?;
    Ok(())
}

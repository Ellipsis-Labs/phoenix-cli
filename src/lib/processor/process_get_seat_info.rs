use crate::helpers::market_helpers::*;
use phoenix_sdk::sdk_client::*;
use phoenix_types as phoenix;
use solana_sdk::pubkey::Pubkey;

pub async fn process_get_seat_info(
    market_pubkey: &Pubkey,
    trader_pubkey: &Pubkey,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    let (seat_address, _) = Pubkey::find_program_address(
        &[b"seat", market_pubkey.as_ref(), trader_pubkey.as_ref()],
        &phoenix::ID,
    );
    println!("Seat address: {}", seat_address);
    let status = get_seat_status(sdk, &seat_address).await;
    match status {
        Ok(status) => println!("Seat status: {}", status),
        _ => println!("Seat status not found"),
    }
    Ok(())
}

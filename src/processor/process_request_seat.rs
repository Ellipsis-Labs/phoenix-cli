use phoenix_sdk::sdk_client::*;
use solana_sdk::pubkey::Pubkey;
use phoenix_types::instructions::create_request_seat_instruction; 


pub async fn process_request_seat(
    market_pubkey: &Pubkey,
    trader_pubkey: &Pubkey,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    let ix = create_request_seat_instruction(&trader_pubkey, &market_pubkey);
    let tx = sdk.client
        .sign_send_instructions(vec![ix], vec![])
        .await;
    
    match tx {
        Ok(tx) => println!("Requested seat, transaction signature: {}", tx),
        Err(e) => println!("Error requesting seat, trader may already have a seat: {}", e),
    }

    Ok(())
}
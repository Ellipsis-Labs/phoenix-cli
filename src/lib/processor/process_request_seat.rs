use phoenix::program::instruction_builders::create_request_seat_instruction;
use phoenix_sdk::sdk_client::*;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub async fn process_request_seat(
    market_pubkey: &Pubkey,
    sdk: &SDKClient,
    prio_fee_instructions: Vec<Instruction>,
) -> anyhow::Result<()> {
    let ix = create_request_seat_instruction(&sdk.core.trader, market_pubkey);
    let ix_to_send = prio_fee_instructions
        .into_iter()
        .chain(vec![ix])
        .collect::<Vec<Instruction>>();
    let tx = sdk.client.sign_send_instructions(ix_to_send, vec![]).await;

    match tx {
        Ok(tx) => println!(
            "Requested seat transaction sent, signature: {} \nCan check transaction status with: phoenix-cli get-transaction {}",
            tx, tx
        ),
        Err(e) => println!("Error requesting seat: {}", e),
    }

    Ok(())
}

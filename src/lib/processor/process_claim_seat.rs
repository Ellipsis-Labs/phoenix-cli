use ellipsis_client::EllipsisClient;
use phoenix_sdk::utils::create_claim_seat_ix_if_needed;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signer::Signer};

pub async fn process_claim_seat(
    client: &EllipsisClient,
    market_pubkey: &Pubkey,
    prio_fee_instructions: Vec<Instruction>,
) -> anyhow::Result<()> {
    let claim_seat_ix =
        create_claim_seat_ix_if_needed(client, market_pubkey, &client.payer.pubkey()).await?;
    println!("Claiming seat for pubkey: {}", client.payer.pubkey());

    if !claim_seat_ix.is_empty() {
        let ix_to_send = prio_fee_instructions
            .into_iter()
            .chain(claim_seat_ix)
            .collect::<Vec<Instruction>>();
        let tx = client.sign_send_instructions(ix_to_send, vec![]).await?;
        println!("Claim seat transaction sent, signature: {} \nCan check transaction status with: phoenix-cli get-transaction {}", tx, tx);
    } else {
        println!("Seat already created for pubkey: {}", client.payer.pubkey());
    }

    Ok(())
}

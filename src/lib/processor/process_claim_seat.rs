use ellipsis_client::EllipsisClient;
use phoenix_sdk::utils::create_claim_seat_ix_if_needed;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

pub async fn process_claim_seat(
    client: &EllipsisClient,
    market_pubkey: &Pubkey,
) -> anyhow::Result<()> {
    let claim_seat_ix =
        create_claim_seat_ix_if_needed(client, market_pubkey, &client.payer.pubkey()).await?;
    println!("Claiming seat for pubkey: {}", client.payer.pubkey());

    if !claim_seat_ix.is_empty() {
        let tx = client.sign_send_instructions(claim_seat_ix, vec![]).await?;
        println!("Claim seat transaction: {}", tx);
    } else {
        println!("Seat already created for pubkey: {}", client.payer.pubkey());
    }

    Ok(())
}

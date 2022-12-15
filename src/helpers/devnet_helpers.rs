use ellipsis_client::EllipsisClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;

// Create_mint on devnet, utilizing devnet-token-faucet
pub async fn find_or_create_devnet_mint(
    client: &EllipsisClient,
    ticker: &str,
    decimals: u8,
) -> anyhow::Result<Pubkey> {
    let (mint, _) = Pubkey::find_program_address(
        &["mint".as_bytes(), ticker.to_lowercase().as_ref()],
        &devnet_token_faucet::id(),
    );
    if client.get_account(&mint).await.is_err() {
        let mint_ix = devnet_token_faucet::create_mint_ix(
            devnet_token_faucet::id(),
            client.payer.pubkey(),
            ticker.to_string(),
            decimals,
        );
        client
            .sign_send_instructions(vec![mint_ix], vec![&client.payer])
            .await?;
    }
    Ok(mint)
}




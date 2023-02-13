use crate::helpers::devnet_helpers::*;
use ellipsis_client::EllipsisClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;

// Only valid for sandbox devnet markets
pub async fn process_mint_tokens(
    client: &EllipsisClient,
    payer: &Keypair,
    recipient_pubkey: &Pubkey,
    mint_ticker: String,
    amount: u64,
) -> anyhow::Result<()> {
    let mut instructions = vec![];

    let mint_pda = find_or_create_devnet_mint(
        client,
        &mint_ticker,
        9, //Decimals only used in creating mint. No effect if mint already exists
    )
    .await?;

    // Get or create the ATA for the recipient. If doesn't exist, create token account
    let recipient_ata =
        spl_associated_token_account::get_associated_token_address(recipient_pubkey, &mint_pda);

    if client.get_account(&recipient_ata).await.is_err() {
        println!("Creating ATA");
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account(
                &payer.pubkey(),
                recipient_pubkey,
                &mint_pda,
                &spl_token::id(),
            ),
        )
    };

    // Call devnet-token-faucet airdrop spl instruction
    instructions.push(devnet_token_faucet::airdrop_spl_with_ticker_ix(
        &devnet_token_faucet::id(),
        mint_ticker,
        recipient_pubkey,
        amount,
    ));

    client
        .sign_send_instructions(instructions, vec![payer])
        .await?;

    println!(
        "{} Tokens minted! Mint pubkey: {},  Recipient address: {}",
        amount, mint_pda, recipient_pubkey
    );

    Ok(())
}

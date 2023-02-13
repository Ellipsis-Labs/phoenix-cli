use phoenix_sdk::sdk_client::*;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use spl_token::state::Mint;

// Only valid for sandbox devnet markets
pub async fn process_mint_tokens_for_market(
    sdk: &SDKClient,
    recipient_pubkey: &Pubkey,
    base_amount: u64,
    quote_amount: u64,
) -> anyhow::Result<()> {
    // Get base and quote mints from market metadata
    let market_metadata = sdk.get_active_market_metadata();
    let base_mint = market_metadata.base_mint;
    let quote_mint = market_metadata.quote_mint;

    let base_mint_account = Mint::unpack(&sdk.client.get_account_data(&base_mint).await?)?;
    let quote_mint_account = Mint::unpack(&sdk.client.get_account_data(&quote_mint).await?)?;

    let quote_mint_authority = quote_mint_account
        .mint_authority
        .ok_or_else(|| anyhow::anyhow!("Quote mint authority is not set. Cannot mint tokens"))?;
    let base_mint_authority = base_mint_account
        .mint_authority
        .ok_or_else(|| anyhow::anyhow!("Base mint authority is not set. Cannot mint tokens"))?;

    if sdk.client.get_account(&quote_mint_authority).await?.owner != devnet_token_faucet::id() {
        return Err(anyhow::anyhow!(
            "Quote mint authority is not owned by devnet-token-faucet"
        ));
    }

    if sdk.client.get_account(&base_mint_authority).await?.owner != devnet_token_faucet::id() {
        return Err(anyhow::anyhow!(
            "Base mint authority is not owned by devnet-token-faucet"
        ));
    }

    // Get or create the ATA for the recipient. If doesn't exist, create token account
    let mut instructions = vec![];

    let recipient_ata_base =
        spl_associated_token_account::get_associated_token_address(recipient_pubkey, &base_mint);

    if sdk.client.get_account(&recipient_ata_base).await.is_err() {
        println!("Creating ATA for base token");
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account(
                &sdk.client.payer.pubkey(),
                recipient_pubkey,
                &base_mint,
                &spl_token::id(),
            ),
        )
    };

    let recipient_ata_quote =
        spl_associated_token_account::get_associated_token_address(recipient_pubkey, &quote_mint);

    if sdk.client.get_account(&recipient_ata_quote).await.is_err() {
        println!("Creating ATA for quote token");
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account(
                &sdk.client.payer.pubkey(),
                recipient_pubkey,
                &quote_mint,
                &spl_token::id(),
            ),
        )
    };

    instructions.push(devnet_token_faucet::airdrop_spl_with_mint_pdas_ix(
        &devnet_token_faucet::id(),
        &base_mint,
        &base_mint_authority,
        recipient_pubkey,
        base_amount,
    ));

    instructions.push(devnet_token_faucet::airdrop_spl_with_mint_pdas_ix(
        &devnet_token_faucet::id(),
        &quote_mint,
        &quote_mint_authority,
        recipient_pubkey,
        quote_amount,
    ));
    let signature = sdk
        .client
        .sign_send_instructions(instructions, vec![])
        .await?;
    println!("Tokens minted! Signature: {}", signature);

    Ok(())
}

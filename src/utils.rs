use crate::TokenRegistry;
use anyhow::anyhow;
use borsh::BorshSerialize;
use ellipsis_client::{EllipsisClient, EllipsisClientResult};
use phoenix::program::get_vault_address;
use phoenix::program::status::SeatApprovalStatus;
use phoenix_sdk::sdk_client::*;
use phoenix_types::market::{MarketHeader, TraderState};
use rand::{rngs::StdRng, SeedableRng};
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::{
    keypair::{read_keypair_file, Keypair},
    Signer,
};
use solana_sdk::{keccak, system_instruction};
use spl_token::state::Mint;
use std::env;

pub fn get_network(network_str: &str) -> &str {
    match network_str {
        "localnet" | "local" => "http://127.0.0.1:8899",
        "devnet" | "dev" => "https://api.devnet.solana.com",
        "mainnet" | "main" | "mainnet-beta" => "https://api.mainnet-beta.solana.com",
        _ => network_str,
    }
}

pub fn is_devnet(network_str: &str) -> bool {
    matches!(network_str, "devnet" | "dev")
}

// TODO: Eventually make this use the devenet token faucet if devnet
pub fn create_mint_ixs(
    context: &EllipsisClient,
    authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
    mint: &Keypair,
) -> EllipsisClientResult<Vec<Instruction>> {
    let ixs = vec![
        system_instruction::create_account(
            &context.payer.pubkey(),
            &mint.pubkey(),
            context.rent_exempt(Mint::LEN),
            Mint::LEN as u64,
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint.pubkey(),
            authority,
            freeze_authority,
            decimals,
        )
        .unwrap(),
    ];
    Ok(ixs)
}

pub fn get_payer_keypair() -> Keypair {
    match env::var("PAYER").is_ok() {
        true => Keypair::from_base58_string(&env::var("PAYER").expect("$PAYER is not set")[..]),
        false => read_keypair_file(&*shellexpand::tilde("~/.config/solana/id.json"))
            .map_err(|e| anyhow!(e.to_string()))
            .unwrap(),
    }
}

pub fn get_discriminant(type_name: &str) -> anyhow::Result<u64> {
    Ok(u64::from_le_bytes(
        keccak::hashv(&[phoenix::id().as_ref(), type_name.as_bytes()]).as_ref()[..8].try_into()?,
    ))
}

pub fn get_all_appoved_seats_for_market(sdk: &SDKClient) -> anyhow::Result<Vec<(Pubkey, Account)>> {
    // Get discriminant for seat account
    let seat_account_discriminant = get_discriminant("phoenix::program::accounts::Seat")?;

    // Get Program Accounts, filtering for the market account discriminant
    let memcmp = RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
        0,
        [
            seat_account_discriminant.to_le_bytes().to_vec(),
            sdk.active_market_key.to_bytes().to_vec(),
        ]
        .concat(),
    ));

    let approved = RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
        72,
        SeatApprovalStatus::Approved.try_to_vec()?,
    ));

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![memcmp, approved]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            commitment: Some(CommitmentConfig::confirmed()),
            ..RpcAccountInfoConfig::default()
        },
        ..RpcProgramAccountsConfig::default()
    };

    let accounts = sdk
        .client
        .get_program_accounts_with_config(&phoenix::id(), config)?;
    Ok(accounts)
}

pub fn get_all_markets(client: &EllipsisClient) -> anyhow::Result<Vec<(Pubkey, Account)>> {
    // Get discriminant for market account
    let market_account_discriminant = get_discriminant("phoenix::program::accounts::MarketHeader")?;

    // Get Program Accounts, filtering for the market account discriminant
    #[allow(deprecated)]
    //Allow deprecated because solana_client struct Memcmp.encoding since 1.11.2. Upgrade workspace package
    let memcmp = RpcFilterType::Memcmp(Memcmp {
        offset: 0,
        bytes: MemcmpEncodedBytes::Bytes(market_account_discriminant.to_le_bytes().to_vec()),
        encoding: None,
    });

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![memcmp]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            commitment: Some(CommitmentConfig::confirmed()),
            ..RpcAccountInfoConfig::default()
        },
        ..RpcProgramAccountsConfig::default()
    };

    let accounts = client.get_program_accounts_with_config(&phoenix::id(), config)?;
    Ok(accounts)
}

pub fn print_market_summary_data(
    market_pubkey: &Pubkey,
    header: &MarketHeader,
    token_registry: &TokenRegistry,
) {
    let base_pubkey = header.base_params.mint_key;
    let base_token = if let Some(ticker) = token_registry.get_ticker_by_pubkey(&base_pubkey) {
        ticker
    } else {
        base_pubkey.to_string()
    };

    let quote_pubkey = header.quote_params.mint_key;
    let quote_token = if let Some(ticker) = token_registry.get_ticker_by_pubkey(&quote_pubkey) {
        ticker
    } else {
        quote_pubkey.to_string()
    };

    println!("--------------------------------------------");
    println!("MARKET: {:?}", market_pubkey);
    println!("Base Token: {:?}", base_token);
    println!("Quote Token: {:?}", quote_token);
    println!("Authority: {:?}", header.authority);
}

pub async fn print_market_details(
    sdk: &SDKClient,
    market_pubkey: &Pubkey,
    market_metadata: &MarketMetadata,
    token_registry: &TokenRegistry,
) {
    let base_pubkey = market_metadata.base_mint;
    let base_token = if let Some(ticker) = token_registry.get_ticker_by_pubkey(&base_pubkey) {
        ticker
    } else {
        base_pubkey.to_string()
    };

    let quote_pubkey = market_metadata.quote_mint;
    let quote_token = if let Some(ticker) = token_registry.get_ticker_by_pubkey(&quote_pubkey) {
        ticker
    } else {
        quote_pubkey.to_string()
    };

    let base_vault = get_vault_address(market_pubkey, &base_pubkey).0;
    let quote_vault = get_vault_address(market_pubkey, &quote_pubkey).0;

    let base_vault_acct =
        spl_token::state::Account::unpack(&sdk.client.get_account(&base_vault).await.unwrap().data)
            .unwrap();

    let quote_vault_acct = spl_token::state::Account::unpack(
        &sdk.client.get_account(&quote_vault).await.unwrap().data,
    )
    .unwrap();

    println!(
        "Base Vault balance: {}",
        get_decimal_string(base_vault_acct.amount, sdk.base_decimals)
    );

    println!(
        "Quote Vault balance: {}",
        get_decimal_string(quote_vault_acct.amount, sdk.quote_decimals)
    );

    println!("Base Token: {}", base_token);
    println!("Quote Token: {}", quote_token);
    println!(
        "Base Lot Size: {} ({})",
        get_decimal_string(market_metadata.base_lot_size, market_metadata.base_decimals),
        base_token
    );

    println!(
        "Quote Lot Size: {} ({})",
        get_decimal_string(
            market_metadata.quote_lot_size,
            market_metadata.quote_decimals
        ),
        quote_token
    );
    println!(
        "Tick size (quote atoms per base unit): {}",
        market_metadata.tick_size_in_quote_atoms_per_base_unit
    );
}

pub fn print_trader_state(sdk: &SDKClient, pubkey: &Pubkey, state: &TraderState) {
    if state.base_lots_locked == 0
        && state.base_lots_free == 0
        && state.quote_lots_locked == 0
        && state.quote_lots_free == 0
    {
        return;
    }
    println!("--------------------------------");
    println!("Trader pubkey: {:?}", pubkey);
    println!(
        "Base lots locked: {}",
        get_decimal_string(
            sdk.base_lots_to_base_amount(state.base_lots_locked),
            sdk.base_decimals
        )
    );
    println!(
        "Base lots free: {}",
        get_decimal_string(
            sdk.base_lots_to_base_amount(state.base_lots_free),
            sdk.base_decimals
        )
    );
    println!(
        "Quote lots locked: {}",
        get_decimal_string(
            sdk.quote_lots_to_quote_amount(state.quote_lots_locked),
            sdk.quote_decimals
        )
    );
    println!(
        "Quote lots free: {}",
        get_decimal_string(
            sdk.quote_lots_to_quote_amount(state.quote_lots_free),
            sdk.quote_decimals
        )
    );
}

// Create_mint with client.payer as the mint authority with static localhost address for testing
pub async fn create_mint(client: &EllipsisClient, decimals: u8) -> anyhow::Result<Pubkey> {
    let mut rng = StdRng::from_entropy();
    let mint_kp = Keypair::generate(&mut rng);
    let instructions = create_mint_ixs(client, &client.payer.pubkey(), None, decimals, &mint_kp)?;
    client
        .sign_send_instructions(instructions, vec![&client.payer, &mint_kp])
        .await?;
    Ok(mint_kp.pubkey())
}

// Create_mint on devnet, utilizing devnet-token-faucet
pub async fn find_or_create_devnet_mint(
    client: &EllipsisClient,
    ticker: &str,
    decimals: u8,
) -> anyhow::Result<Pubkey> {
    let mint_ix = devnet_token_faucet::create_mint_ix(
        devnet_token_faucet::id(),
        client.payer.pubkey(),
        ticker.to_string(),
        decimals,
    );

    let (mint, _) = Pubkey::find_program_address(
        &["mint".as_bytes(), ticker.to_lowercase().as_ref()],
        &devnet_token_faucet::id(),
    );

    let mut instructions = vec![];
    if client.get_account(&mint).await.is_err() {
        instructions.push(mint_ix);
        client
            .sign_send_instructions(instructions, vec![&client.payer])
            .await?;
    }
    Ok(mint)
}

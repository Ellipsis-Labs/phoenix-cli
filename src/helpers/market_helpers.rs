use borsh::BorshDeserialize;
use ellipsis_client::EllipsisClient;
use phoenix_sdk::sdk_client::*;
use phoenix_types::dispatch::load_with_dispatch;
use phoenix_types::market::Ladder;
use phoenix_types as phoenix; 
use phoenix_types::market::MarketHeader;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::keccak;
use solana_sdk::pubkey::Pubkey;
use std::mem::size_of;

pub fn get_discriminant(type_name: &str) -> anyhow::Result<u64> {
    Ok(u64::from_le_bytes(
        keccak::hashv(&[phoenix::id().as_ref(), type_name.as_bytes()]).as_ref()[..8].try_into()?,
    ))
}

pub async fn get_seat_status(
    sdk: &SDKClient,
    seat_key: &Pubkey,
) -> anyhow::Result<phoenix_types::market::SeatApprovalStatus> {
    // Get seat account and deserialize
    let seat_acc = sdk.client.get_account(&seat_key).await?;
    let mut seat_acc_data = seat_acc.data.to_vec();
    let (_, seat_approval_bytes) = seat_acc_data.split_at_mut(72);
    let status_as_u64  = u64::try_from_slice(&seat_approval_bytes[0..8])?;
    let seat_status = phoenix_types::market::SeatApprovalStatus::from(status_as_u64);
    Ok(seat_status)
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

pub async fn get_book_levels(
    market_pubkey: &Pubkey,
    client: &EllipsisClient,
    levels: u64,
) -> anyhow::Result<Ladder> {
    // Get market account
    let mut market_account_data = client.get_account_data(&market_pubkey).await?;
    let (header_bytes, market_bytes) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header = MarketHeader::try_from_slice(header_bytes)?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch(&header.market_size_params, market_bytes)
        .unwrap()
        .inner;

    Ok(market.get_ladder(levels))
}
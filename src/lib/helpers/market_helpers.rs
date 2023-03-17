use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use ellipsis_client::EllipsisClient;
use phoenix::program::{load_with_dispatch, status::SeatApprovalStatus, MarketHeader};
use phoenix::state::markets::FIFOOrderId;
use phoenix::state::markets::FIFORestingOrder;
use phoenix::state::markets::{Ladder, Market};
use phoenix::state::OrderPacket;

use phoenix_sdk::sdk_client::*;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};
use solana_sdk::account::Account;
use solana_sdk::clock::Clock;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::keccak;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::sysvar;
use std::collections::BTreeMap;
use std::mem::size_of;

pub fn get_discriminant(type_name: &str) -> anyhow::Result<u64> {
    Ok(u64::from_le_bytes(
        keccak::hashv(&[phoenix::id().as_ref(), type_name.as_bytes()]).as_ref()[..8].try_into()?,
    ))
}

pub async fn get_seat_status(
    sdk: &SDKClient,
    seat_key: &Pubkey,
) -> anyhow::Result<SeatApprovalStatus> {
    // Get seat account and deserialize
    let seat_acc = sdk.client.get_account(seat_key).await?;
    let mut seat_acc_data = seat_acc.data.to_vec();
    let (_, seat_approval_bytes) = seat_acc_data.split_at_mut(72);
    let status_as_u64 = u64::try_from_slice(&seat_approval_bytes[0..8])?;
    let seat_status = SeatApprovalStatus::from(status_as_u64);
    Ok(seat_status)
}

pub async fn get_all_markets(client: &EllipsisClient) -> anyhow::Result<Vec<(Pubkey, Account)>> {
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

    let accounts = client
        .get_program_accounts_with_config(&phoenix::id(), config)
        .await?;
    Ok(accounts)
}

// Get each trader's trader index and map to the trader's pubkey
pub fn get_all_registered_traders(
    market: &dyn Market<Pubkey, FIFOOrderId, FIFORestingOrder, OrderPacket>,
) -> BTreeMap<u64, Pubkey> {
    let mut trader_index_to_pubkey = BTreeMap::new();
    market
        .get_registered_traders()
        .iter()
        .map(|(trader, _)| *trader)
        .for_each(|trader| {
            trader_index_to_pubkey.insert(market.get_trader_index(&trader).unwrap() as u64, trader);
        });
    trader_index_to_pubkey
}

pub async fn get_all_seats(client: &EllipsisClient) -> anyhow::Result<Vec<(Pubkey, Account)>> {
    // Get discriminant for seat account
    let seat_account_discriminant = get_discriminant("phoenix::program::accounts::Seat")?;

    #[allow(deprecated)]
    let memcmp = RpcFilterType::Memcmp(Memcmp {
        offset: 0,
        bytes: MemcmpEncodedBytes::Bytes(seat_account_discriminant.to_le_bytes().to_vec()),
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

    let accounts = client
        .get_program_accounts_with_config(&phoenix::id(), config)
        .await?;

    Ok(accounts)
}

pub async fn get_book_levels(
    market_pubkey: &Pubkey,
    client: &EllipsisClient,
    levels: u64,
) -> anyhow::Result<Ladder> {
    // Get market account
    let mut market_and_clock = client
        .get_multiple_accounts_with_commitment(
            &[*market_pubkey, sysvar::clock::id()],
            CommitmentConfig::confirmed(),
        )
        .await?
        .value;

    let market_account_data = market_and_clock
        .remove(0)
        .ok_or(anyhow::Error::msg("Market account not found"))?
        .data;

    let clock_account_data = market_and_clock
        .remove(0)
        .ok_or(anyhow::Error::msg("Clock account not found"))?
        .data;

    let clock: Clock = bincode::deserialize(&clock_account_data)
        .map_err(|_| anyhow::Error::msg("Error deserializing clock"))?;

    let (header_bytes, market_bytes) = market_account_data.split_at(size_of::<MarketHeader>());
    let header: &MarketHeader = bytemuck::try_from_bytes(header_bytes)
        .map_err(|e| anyhow::anyhow!("Error getting market header. Error: {:?}", e))?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch(&header.market_size_params, market_bytes)?.inner;

    Ok(market.get_ladder_with_expiration(
        levels,
        Some(clock.slot),
        Some(clock.unix_timestamp as u64),
    ))
}

pub async fn get_all_approved_seats_for_market(
    sdk: &SDKClient,
    market: &Pubkey,
) -> anyhow::Result<Vec<(Pubkey, Account)>> {
    // Get discriminant for seat account
    let seat_account_discriminant = get_discriminant("phoenix::program::accounts::Seat")?;

    // Get Program Accounts, filtering for the market account discriminant
    let memcmp = RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
        0,
        [
            seat_account_discriminant.to_le_bytes().to_vec(),
            market.to_bytes().to_vec(),
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
        .get_program_accounts_with_config(&phoenix::id(), config)
        .await?;
    Ok(accounts)
}

pub async fn get_market_header(
    sdk: &SDKClient,
    market_pubkey: &Pubkey,
) -> anyhow::Result<MarketHeader> {
    let market_account_data = sdk.client.get_account_data(market_pubkey).await?;
    let (header_bytes, _market_bytes) = market_account_data.split_at(size_of::<MarketHeader>());
    let header: &MarketHeader = bytemuck::try_from_bytes(header_bytes)
        .map_err(|e| anyhow::anyhow!("Error getting market header. Error: {:?}", e))?;

    Ok(*header)
}

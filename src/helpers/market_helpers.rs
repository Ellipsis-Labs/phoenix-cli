use anyhow::Ok;
use borsh::BorshDeserialize;
use ellipsis_client::EllipsisClient;
use phoenix_sdk::sdk_client::*;
use phoenix_types as phoenix;
use phoenix_types::dispatch::load_with_dispatch;
use phoenix_types::market::Ladder;
use phoenix_types::market::MarketHeader;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};
use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::keccak;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::mem::size_of;
use std::str::FromStr;

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
    let seat_acc = sdk.client.get_account(seat_key).await?;
    let mut seat_acc_data = seat_acc.data.to_vec();
    let (_, seat_approval_bytes) = seat_acc_data.split_at_mut(72);
    let status_as_u64 = u64::try_from_slice(&seat_approval_bytes[0..8])?;
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
    let mut market_account_data = client.get_account_data(market_pubkey).await?;
    let (header_bytes, market_bytes) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header = MarketHeader::try_from_slice(header_bytes)?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch(&header.market_size_params, market_bytes)
        .ok_or_else(|| anyhow::anyhow!("Failed to load market"))?
        .inner;

    Ok(market.get_ladder(levels))
}

pub async fn get_historical_signatures(
    market_pubkey: &Pubkey,
    trader_pubkey: &Pubkey,
    lookback_slots: u64,
    sdk: &SDKClient,
) -> anyhow::Result<Vec<Signature>> {
    let current_slot = sdk.client.get_slot()?;
    let mut target_slot = current_slot - lookback_slots;
    let trader_transactions_with_signature =
        get_transactions_with_signature(trader_pubkey, target_slot, sdk).await?;
    if trader_transactions_with_signature.is_empty() {
        return Ok(Vec::new());
    }

    let last_trader_slot = trader_transactions_with_signature.last().unwrap().slot;
    if last_trader_slot > target_slot {
        target_slot = last_trader_slot;
    }
    let market_transactions_with_signature =
        get_transactions_with_signature(market_pubkey, target_slot, sdk).await?;
    if market_transactions_with_signature.is_empty() {
        println!("No events found for this market");
        return Ok(Vec::new());
    }
    // Filter down to only the signatures in each transaction
    let trader_signatures: Vec<Signature> = trader_transactions_with_signature
        .iter()
        .map(|x| Signature::from_str(&x.signature).unwrap())
        .collect();
    let market_signatures: HashSet<Signature, RandomState> = HashSet::from_iter(
        market_transactions_with_signature
            .iter()
            .map(|x| Signature::from_str(&x.signature).unwrap()),
    );

    // create a vector of signatures that are the intersection of market and trader transactions
    let joint_signatures: Vec<Signature> = trader_signatures
        .into_iter()
        .filter(|x| market_signatures.contains(x))
        .collect();

    Ok(joint_signatures)
}

pub async fn get_historical_market_signatures_exluding_trader(
    market_pubkey: &Pubkey,
    trader_pubkey: &Pubkey,
    lookback_slots: u64,
    sdk: &SDKClient,
) -> anyhow::Result<Vec<Signature>> {
    let current_slot = sdk.client.get_slot()?;
    let target_slot = current_slot - lookback_slots;
    let trader_transactions_with_signature =
        get_transactions_with_signature(trader_pubkey, target_slot, sdk).await?;
    let market_transactions_with_signature =
        get_transactions_with_signature(market_pubkey, target_slot, sdk).await?;
    if market_transactions_with_signature.is_empty() {
        println!("No events found for this market");
        return Ok(Vec::new());
    }

    // Filter down to only the signatures in each transaction
    let trader_signatures: HashSet<Signature, RandomState> = HashSet::from_iter(
        trader_transactions_with_signature
            .iter()
            .map(|x| Signature::from_str(&x.signature).unwrap()),
    );
    let market_signatures: Vec<Signature> = market_transactions_with_signature
        .iter()
        .map(|x| Signature::from_str(&x.signature).unwrap())
        .collect();

    // create a vector of signatures of all market events not initiated by the trader
    let joint_signatures: Vec<Signature> = market_signatures
        .into_iter()
        .filter(|x| !trader_signatures.contains(x))
        .collect();

    Ok(joint_signatures)
}

pub async fn get_transactions_with_signature(
    pubkey: &Pubkey,
    min_slot: u64,
    sdk: &SDKClient,
) -> anyhow::Result<Vec<RpcConfirmedTransactionStatusWithSignature>> {
    let mut transactions_with_signature = vec![];

    let mut config = GetConfirmedSignaturesForAddress2Config {
        before: None,
        until: None,
        limit: None,
        commitment: None,
    };
    loop {
        let next_signatures = sdk
            .client
            .get_signatures_for_address_with_config(pubkey, config)?;
        if next_signatures.last().is_none() {
            break;
        }
        let last_signature = next_signatures.last().unwrap();
        if last_signature.slot < min_slot {
            // append all signatures before min slot
            transactions_with_signature.extend(
                next_signatures
                    .into_iter()
                    .take_while(|sig| sig.slot >= min_slot),
            );
            break;
        }

        config = GetConfirmedSignaturesForAddress2Config {
            before: Some(Signature::from_str(&last_signature.signature).unwrap()),
            until: None,
            limit: None,
            commitment: None,
        };
        transactions_with_signature.extend(next_signatures);
    }

    Ok(transactions_with_signature)
}

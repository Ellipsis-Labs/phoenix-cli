use anyhow::Ok;
use borsh::BorshDeserialize;
use ellipsis_client::EllipsisClient;
use phoenix_sdk::sdk_client::*;
use phoenix_types as phoenix;
use phoenix_types::dispatch::load_with_dispatch;
use phoenix_types::market::Ladder;
use phoenix_types::market::MarketHeader;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
use solana_sdk::keccak;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use solana_sdk::signature::Signature;
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

pub async fn get_transaction_history(
    market_pubkey: &Pubkey,
    trader_pubkey: &Pubkey,
    slots_back: u64, 
    sdk: &SDKClient,   
)  -> anyhow::Result<Vec<Signature>>{
    // goal is to return the trades over last day for a given market and trader
    // will appoximate a day by slot time, approx 2 slots/second and 86400 seconds/day = 172800 slots/day
    let current_slot = sdk.client.get_slot()?;
    let mut target_slot = current_slot - slots_back;
    let trader_signatures = get_signatures(trader_pubkey, target_slot, sdk).await?;
    if trader_signatures.is_empty() {
        println!("No trades found for this trader");
        return Ok(Vec::new());
    }

    let last_trader_slot = trader_signatures.last().unwrap().slot;
    if last_trader_slot > target_slot {
        target_slot = last_trader_slot;
    }
    let market_signatures = get_signatures(market_pubkey, target_slot, sdk).await?; 
    if market_signatures.is_empty() {
        println!("No trades found for this market");
        return Ok(Vec::new());
    }
    // create a vector of signatures that are in both the trader and market vectors
    let joint_signatures: Vec<&RpcConfirmedTransactionStatusWithSignature>  = trader_signatures.iter().filter(|x| market_signatures.contains(x)).collect();
    let joint_signatures_filtered: Vec<Signature> = joint_signatures.iter().map(|x| Signature::from_str(&x.signature).unwrap()).collect();

    Ok(joint_signatures_filtered)
}

pub async fn get_signatures( 
    pubkey: &Pubkey,
    min_slot: u64,
    sdk: &SDKClient,
) -> anyhow::Result<Vec<RpcConfirmedTransactionStatusWithSignature>>{ 
    let mut signatures = vec![];

    let mut config = GetConfirmedSignaturesForAddress2Config{ 
        before: None,
        until: None,
        limit: None,
        commitment: None 
    };
    loop {
        let next_signatures = sdk.client.get_signatures_for_address_with_config(pubkey, config)?;
        if next_signatures.last().is_none() {
            break;
        }
        let last_signature = next_signatures.last().unwrap();
        if last_signature.slot < min_slot {
            // append all signatures before min slot 
            signatures.extend(next_signatures.into_iter().take_while(|sig| sig.slot >= min_slot));
            break;
        }

        config = GetConfirmedSignaturesForAddress2Config{ 
            before: Some(Signature::from_str(&last_signature.signature).unwrap()),
            until: None,
            limit: None,
            commitment: None 
        };
        signatures.extend(next_signatures);

    }

    Ok(signatures)
}


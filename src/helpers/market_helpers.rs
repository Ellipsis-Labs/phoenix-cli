use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use ellipsis_client::EllipsisClient;
use phoenix::program::status::SeatApprovalStatus;
use phoenix_sdk::sdk_client::*;
use phoenix_types::dispatch::load_with_dispatch_mut;
use phoenix_types::market::Ladder;
use phoenix_types::enums::Side;
use phoenix_types::market::LadderOrder;
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

pub fn get_seat_status(
    sdk: &SDKClient,
    seat_key: &Pubkey,
) -> anyhow::Result<phoenix::program::status::SeatApprovalStatus> {
    // Get discriminant for seat account
    let seat_account_discriminant = get_discriminant("phoenix::program::accounts::Seat")?;

    let statuses = vec![
        SeatApprovalStatus::Retired,
        SeatApprovalStatus::Approved,
        SeatApprovalStatus::NotApproved,
    ];

    for status in statuses {
        // Get Program Accounts, filtering for the market account discriminant
        let memcmp = RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
            0,
            [
                seat_account_discriminant.to_le_bytes().to_vec(),
                sdk.active_market_key.to_bytes().to_vec(),
            ]
            .concat(),
        ));

        let status_filter = RpcFilterType::Memcmp(Memcmp::new_raw_bytes(72, status.try_to_vec()?));

        let config = RpcProgramAccountsConfig {
            filters: Some(vec![memcmp, status_filter]),
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

        for (key, _) in accounts {
            if key == *seat_key {
                return Ok(status);
            }
        }
    }

    Ok(SeatApprovalStatus::NotApproved)
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
    let market = load_with_dispatch_mut(&header.market_size_params, market_bytes)
        .unwrap()
        .inner;

    Ok(market.get_ladder(levels))
}

pub async fn get_full_orderbook(
    market_pubkey: &Pubkey,
    client: &EllipsisClient,
) -> anyhow::Result<Ladder> {
    // Get market account
    let mut market_account_data = client.get_account_data(&market_pubkey).await?;
    let (header_bytes, market_bytes) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header = MarketHeader::try_from_slice(header_bytes)?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch_mut(&header.market_size_params, market_bytes)
        .unwrap()
        .inner;

    let book_bids  = market.get_book(Side::Bid);
    let book_asks = market.get_book(Side::Ask);
    let mut bids = vec![];
    let mut asks = vec![]; 
    for (order_id, order) in book_bids.iter(){ 
        bids.push(LadderOrder{
            price_in_ticks: order_id.price_in_ticks,
            size_in_base_lots: order.num_base_lots,
        });
    }
    for (order_id, order) in book_asks.iter(){ 
        asks.push(LadderOrder{
            price_in_ticks: order_id.price_in_ticks,
            size_in_base_lots: order.num_base_lots,
        });
    }
    Ok(Ladder { bids: bids, asks: asks })
}



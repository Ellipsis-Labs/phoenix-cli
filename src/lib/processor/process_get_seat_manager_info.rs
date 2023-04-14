use std::mem::size_of;

use ellipsis_client::EllipsisClient;
use phoenix::program::MarketHeader;
use phoenix_seat_manager::{get_seat_manager_address, seat_manager::SeatManager};
use solana_sdk::pubkey::Pubkey;

use crate::helpers::market_helpers::get_seat_manager_data_with_market;

pub async fn process_get_seat_manager_info(
    client: &EllipsisClient,
    market_pubkey: &Pubkey,
) -> anyhow::Result<()> {
    let seat_manager_address = get_seat_manager_address(market_pubkey).0;
    let market_data = client.get_account_data(market_pubkey).await?;
    let market_header =
        bytemuck::from_bytes::<MarketHeader>(market_data.split_at(size_of::<MarketHeader>()).0);
    if market_header.authority != seat_manager_address {
        println!(
            "Authority for Market {} is not the seat manager.",
            market_pubkey
        );
        println!("Market authority: {}", market_header.authority);
        println!("Seat manager address: {}", seat_manager_address);
        return Ok(());
    }
    let seat_manager_info = get_seat_manager_data_with_market(client, market_pubkey).await?;
    print_seat_manager_struct(&seat_manager_info, &seat_manager_address);
    Ok(())
}

pub fn print_seat_manager_struct(seat_manager: &SeatManager, seat_manager_pubkey: &Pubkey) {
    println!("Seat Manager Address: {}", seat_manager_pubkey);
    println!("SM Market: {}", seat_manager.market);
    println!("SM Authority: {}", seat_manager.authority);
    println!("SM Successor: {}", seat_manager.successor);
    println!(
        "Number of designated market makers: {}",
        seat_manager.num_makers
    );

    let dmms: Vec<&Pubkey> = seat_manager
        .designated_market_makers
        .iter()
        .filter(|&&dmm| dmm != Pubkey::default())
        .collect();
    if !dmms.is_empty() {
        println!("DMMs: {:?}", dmms);
    }
}

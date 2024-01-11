use std::mem::size_of;

use ellipsis_client::EllipsisClient;
use phoenix::program::MarketHeader;
use phoenix_sdk::utils::get_evictable_trader_ix;
use phoenix_seat_manager::instruction_builders::{
    create_evict_seat_instruction, EvictTraderAccountBackup,
};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub async fn process_evict_seat(
    client: &EllipsisClient,
    market_pubkey: &Pubkey,
    trader_to_evict: &Option<Pubkey>,
    prio_fee_instructions: Vec<Instruction>,
) -> anyhow::Result<()> {
    let market_bytes = client.get_account_data(market_pubkey).await?;
    let (header_bytes, _market_bytes) = market_bytes.split_at(size_of::<MarketHeader>());
    let market_header = bytemuck::try_from_bytes::<MarketHeader>(header_bytes)
        .map_err(|e| anyhow::anyhow!("Error deserializing market header. Error: {:?}", e))?;

    let maybe_evict_trader_ix = if let Some(trader_pubkey) = trader_to_evict {
        let evict_trader_state = EvictTraderAccountBackup {
            trader_pubkey: *trader_pubkey,
            base_token_account_backup: None,
            quote_token_account_backup: None,
        };
        Some(create_evict_seat_instruction(
            market_pubkey,
            &market_header.base_params.mint_key,
            &market_header.quote_params.mint_key,
            trader_pubkey,
            vec![evict_trader_state],
        ))
    } else {
        get_evictable_trader_ix(client, market_pubkey).await?
    };

    if let Some(evict_trader_ix) = maybe_evict_trader_ix {
        println!("Evicting trader: {}", evict_trader_ix.accounts[13].pubkey);
        let ix_to_send = prio_fee_instructions
            .into_iter()
            .chain(vec![evict_trader_ix])
            .collect::<Vec<Instruction>>();
        let tx = client.sign_send_instructions(ix_to_send, vec![]).await?;
        println!("Evict trader tx: {}", tx);
    } else {
        println!("Cannot evict a trader when the market's trader state is not full.");
        return Ok(());
    }

    Ok(())
}

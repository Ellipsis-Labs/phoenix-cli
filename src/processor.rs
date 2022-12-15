use crate::helpers::print_helpers::*;
use crate::helpers::market_helpers::*;
use crate::helpers::devnet_helpers::*;
use borsh::BorshDeserialize;
use ellipsis_client::EllipsisClient;
use phoenix::program::status::MarketStatus;
use phoenix_sdk::sdk_client::*;
use phoenix_types::dispatch::load_with_dispatch_mut;
use phoenix_types::enums::Side;
use phoenix_types::market::MarketHeader;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signature;
use solana_sdk::signature::Signer;
use spl_token::state::Mint;
use std::mem::size_of;

pub async fn process_get_market(market_pubkey: &Pubkey, sdk: &SDKClient) -> anyhow::Result<()> {
    let market_metadata = sdk.get_active_market_metadata();

    let mut market_account_data = sdk.client.get_account_data(&market_pubkey).await?;
    let (header_bytes, market_bytes) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header = MarketHeader::try_from_slice(header_bytes)?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch_mut(&header.market_size_params, market_bytes)
        .unwrap()
        .inner;

    let taker_fees = market.get_taker_bps();

    print_market_details(&sdk, &market_pubkey, market_metadata, taker_fees).await;
    Ok(())
}

pub fn process_get_all_markets(client: &EllipsisClient) -> anyhow::Result<()> {
    let accounts = get_all_markets(&client)?;

    println!("Found {} market(s)", accounts.len());

    //Deserialize market accounts and print summary information
    for (market_pubkey, mut market_account) in accounts {
        let (header_bytes, _market_bytes) =
            market_account.data.split_at_mut(size_of::<MarketHeader>());

        let header = MarketHeader::try_from_slice(header_bytes)?;

        print_market_summary_data(&market_pubkey, &header);
    }
    Ok(())
}

pub async fn process_get_traders_for_market(
    market_pubkey: &Pubkey,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    // Get market account
    let mut market_account_data = sdk.client.get_account_data(&market_pubkey).await?;
    let (header_bytes, market_bytes) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header = MarketHeader::try_from_slice(header_bytes)?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch_mut(&header.market_size_params, market_bytes)
        .unwrap()
        .inner;

    // Print trader information
    market
        .get_registered_traders()
        .iter()
        .for_each(|(pubkey, state)| {
            print_trader_state(&sdk, pubkey, state);
        });

    Ok(())
}

pub async fn process_get_top_of_book(
    market_pubkey: &Pubkey,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    let ladder = get_book_levels(&market_pubkey, &sdk.client, 1).await?;

    if !ladder.bids.is_empty() {
        println!(
            "Top of book bid price: {}",
            sdk.core.ticks_to_float_price(ladder.bids[0].price_in_ticks)
        );
    }
    if !ladder.asks.is_empty() {
        println!(
            "Top of book ask price: {}",
            sdk.core.ticks_to_float_price(ladder.asks[0].price_in_ticks)
        );
    } else {
        println!("Book is empty");
    }

    Ok(())
}

pub async fn process_get_book(
    market_pubkey: &Pubkey,
    sdk: &SDKClient,
    levels: u64,
) -> anyhow::Result<()> {
    let book = get_book_levels(&market_pubkey, &sdk.client, levels).await?;
    println!("First {} levels of the orderbook: {:?}", levels, book);
    Ok(())
}

pub async fn process_get_full_book(
    market_pubkey: &Pubkey,
    sdk: &SDKClient,
) -> anyhow::Result<()> {

    let book = get_full_orderbook(&market_pubkey, &sdk.client).await?;
    println!("Full orderbook: {:?}", book);
    Ok(())
}

pub async fn process_get_transaction(signature: &Signature, sdk: &SDKClient) -> anyhow::Result<()> {
    let events = sdk.parse_events_from_transaction(signature).await.unwrap();
    log_market_events(&sdk, events);
    Ok(())
}

pub async fn process_get_market_status(
    market_pubkey: &Pubkey,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    // Get market account
    let mut market_account_data = sdk.client.get_account_data(&market_pubkey).await?;
    let (header_bytes, _) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header = MarketHeader::try_from_slice(header_bytes)?;

    let status = MarketStatus::from(header.status);
    println!("Market status: {}", status);
    Ok(())
}

pub async fn process_get_seat_info(
    market_pubkey: &Pubkey,
    trader_pubkey: &Pubkey,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    let (seat_address, _) = Pubkey::find_program_address(
        &[b"seat", market_pubkey.as_ref(), trader_pubkey.as_ref()],
        &phoenix::ID,
    );
    println!("Seat address: {}", seat_address);
    let status = get_seat_status(&sdk, &seat_address);
    match status {
        Ok(status) => println!("Seat status: {}", status),
        _ => println!("Seat status not found"),
    }
    Ok(())
}

pub async fn process_get_open_orders(
    market_pubkey: &Pubkey,
    trader_pubkey: &Pubkey,
    sdk: &SDKClient,
) -> anyhow::Result<()> {
    // Get market account
    let mut market_account_data = sdk.client.get_account_data(&market_pubkey).await?;
    let (header_bytes, market_bytes) = market_account_data.split_at_mut(size_of::<MarketHeader>());
    let header = MarketHeader::try_from_slice(header_bytes)?;

    // Derserialize data and load into correct type
    let market = load_with_dispatch_mut(&header.market_size_params, market_bytes)
        .unwrap()
        .inner;

    let trader_index = market.get_trader_index(&trader_pubkey);

    if let None = trader_index {
        println!("Trader not found");
        return Ok(());
    };

    let book_bids = market.get_book(Side::Bid);
    let book_asks = market.get_book(Side::Ask);

    let mut open_bids = vec![];
    for (order_id, order) in book_bids.iter() {
        if order.trader_index as u32 == trader_index.unwrap() {
            open_bids.push((
                sdk.ticks_to_float_price(order_id.price_in_ticks),
                order.num_base_lots as f64 * sdk.base_lots_to_base_units_multiplier(),
            ));
        }
    }
    println!("Open bids: {:?}", open_bids);

    let mut open_asks = vec![];
    for (order_id, order) in book_asks.iter() {
        if order.trader_index as u32 == trader_index.unwrap() {
            open_asks.push((
                sdk.ticks_to_float_price(order_id.price_in_ticks),
                order.num_base_lots as f64 * sdk.base_lots_to_base_units_multiplier(),
            ));
        }
    }
    println!("Open asks: {:?}", open_asks);

    Ok(())
}

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
        spl_associated_token_account::get_associated_token_address(&recipient_pubkey, &mint_pda);

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
        &recipient_pubkey,
        amount,
    ));

    client
        .sign_send_instructions(instructions, vec![&payer])
        .await?;

    println!(
        "{} Tokens minted! Mint pubkey: {},  Recipient address: {}",
        amount, mint_pda, recipient_pubkey
    );

    Ok(())
}

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

    let quote_mint_authority = quote_mint_account.mint_authority.unwrap();
    let base_mint_authority = base_mint_account.mint_authority.unwrap();

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

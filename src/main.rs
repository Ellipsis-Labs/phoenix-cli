mod token_registry;
mod utils;
use crate::token_registry::TokenRegistry;
use crate::utils::*;

use borsh::BorshDeserialize;
use clap::Parser;
use ellipsis_client::EllipsisClient;
use phoenix::program::instruction_builders::{
    create_change_seat_status_instruction, create_request_seat_authorized_instruction,
};
use phoenix::program::status::{MarketStatus, SeatApprovalStatus};
use phoenix_sdk::sdk_client::*;
use phoenix_types::dispatch::load_with_dispatch_mut;
use phoenix_types::enums::Side;
use phoenix_types::market::MarketHeader;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::signer::Signer;
use spl_token::state::Mint;
use std::mem::size_of;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[clap(short, long, default_value = "local")]
    network: String,
    #[clap(short, long)]
    use_faucet: bool,
    #[clap(subcommand)]
    command: Command,
}

// #[clap(author, version, about)]
#[derive(Debug, Clone, Parser)]
enum Command {
    /// Get summary information on all markets
    GetAllMarkets,
    /// Get information on markets
    GetMarket {
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
    },
    /// Get active traders for a given market
    GetTradersForMarket {
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
    },
    /// Get the best bid and best ask, for a given market
    GetTopOfBook {
        #[clap(short, long, required = true)]
        pubkey: Pubkey,
    },
    /// Mint tokens to a recipient for a given ticker string. Default amount is 100_000_000_000
    // TODO: change this to use the token's decimals (deserialize mint pda data)
    MintTokens {
        #[clap(short, long, required = true)]
        mint_ticker: String,
        #[clap(short, long, required = true)]
        recipient_pubkey: Pubkey,
        #[clap(short, long, required = false, default_value = "100000000000")]
        amount: u64,
    },
    /// Mint both base and quote tokens to a recipient for a given market. Default amounts are 100_000_000_000 for base and 100_000_000 for quote.
    MintTokensForMarket {
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
        #[clap(short, long, required = true)]
        recipient_pubkey: Pubkey,
        #[clap(short, long, required = false, default_value = "100000000000")]
        base_amount: u64,
        #[clap(short, long, required = false, default_value = "100000000")]
        quote_amount: u64,
    },
    GetTransaction {
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
        #[clap(short, long, required = true)]
        signature: Signature,
    },
    /// Get the current status of a market
    GetMarketStatus {
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let network_url = get_network(&cli.network);

    let payer = get_payer_keypair();

    // Get local token registry
    let token_registry_path = PathBuf::from(r"registry.txt");
    let mut token_registry = TokenRegistry::open(&token_registry_path)?;

    println!("Current payer: {}", payer.pubkey());
    println!("Current network: {}", network_url);

    match cli.command {
        Command::GetMarket { market_pubkey } => {
            let sdk_client = SDKClient::new(&market_pubkey, &payer, network_url).await;
            let market_metadata = sdk_client.get_active_market_metadata();
            print_market_details(
                &sdk_client,
                &market_pubkey,
                market_metadata,
                &token_registry,
            )
            .await;
            Ok(())
        }
        Command::GetAllMarkets => {
            let client = EllipsisClient::from_rpc(
                RpcClient::new_with_commitment(network_url, CommitmentConfig::confirmed()),
                &payer,
            )?;

            let accounts = get_all_markets(&client)?;

            println!("Found {} market(s)", accounts.len());

            //Deserialize market accounts and print summary information
            for (market_pubkey, mut market_account) in accounts {
                let (header_bytes, _market_bytes) =
                    market_account.data.split_at_mut(size_of::<MarketHeader>());

                let header = MarketHeader::try_from_slice(header_bytes)?;

                print_market_summary_data(&market_pubkey, &header, &token_registry);
            }
            Ok(())
        }
        Command::GetTradersForMarket { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;

            // Get market account
            let mut market_account_data = sdk.client.get_account_data(&market_pubkey).await?;
            let (header_bytes, market_bytes) =
                market_account_data.split_at_mut(size_of::<MarketHeader>());
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
        Command::GetTopOfBook { pubkey } => {
            let client = EllipsisClient::from_rpc(
                RpcClient::new_with_commitment(network_url, CommitmentConfig::confirmed()),
                &payer,
            )?;

            let sdk = SDKClient::new(&pubkey, &payer, network_url).await;

            // Get market account
            let mut market_account_data = client.get_account_data(&pubkey).await?;
            let (header_bytes, market_bytes) =
                market_account_data.split_at_mut(size_of::<MarketHeader>());
            let header = MarketHeader::try_from_slice(header_bytes)?;

            // Derserialize data and load into correct type
            let market = load_with_dispatch_mut(&header.market_size_params, market_bytes)
                .unwrap()
                .inner;

            let ladder = market.get_ladder(5);

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
                println!("No bids or asks currently");
            }

            Ok(())
        }
        Command::MintTokens {
            mint_ticker,
            recipient_pubkey,
            amount,
        } => {
            if cli.use_faucet {
                let client = EllipsisClient::from_rpc(
                    RpcClient::new_with_commitment(network_url, CommitmentConfig::confirmed()),
                    &payer,
                )?;
                let mut instructions = vec![];

                let mint_pda = find_or_create_devnet_mint(
                    &client,
                    &mint_ticker,
                    9, //Decimals only used in creating mint. No effect if mint already exists
                )
                .await?;

                // Get or create the ATA for the recipient. If doesn't exist, create token account
                let recipient_ata = spl_associated_token_account::get_associated_token_address(
                    &recipient_pubkey,
                    &mint_pda,
                );

                if client.get_account(&recipient_ata).await.is_err() {
                    println!("Error retrieving ATA. Creating ATA");
                    instructions.push(
                        spl_associated_token_account::instruction::create_associated_token_account(
                            &payer.pubkey(),
                            &recipient_pubkey,
                            &mint_pda,
                            &spl_token::id(),
                        ),
                    )
                };

                // Call devnet-token-faucet airdrop spl instruction
                // TODO: rename parameter name from payer to recipent in devnet-token-faucet (payer doesn't need to sign)
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
                    "Tokens minted! Mint pukey: {},  Recipient address: {}",
                    mint_pda, recipient_pubkey
                );
                Ok(())
            } else {
                // TODO: Define localhost behavior
                // If localhost, just create mint and mint to
                println!("Localhost behavior not yet implemented");
                Ok(())
            }
        }
        Command::MintTokensForMarket {
            market_pubkey,
            recipient_pubkey,
            base_amount,
            quote_amount,
        } => {
            if !cli.use_faucet {
                println!("Localhost behavior not yet implemented");
                return Ok(());
            }

            // Get base and quote mints from market metadata
            let sdk_client = SDKClient::new(&market_pubkey, &payer, network_url).await;
            let instructions =
                create_airdrop_spl_ixs(&sdk_client, &recipient_pubkey, base_amount, quote_amount)
                    .await?;

            let signature = sdk_client
                .client
                .sign_send_instructions(instructions, vec![])
                .await?;
            println!("Tokens minted! Signature: {}", signature);

            Ok(())
        }
        Command::GetTransaction {
            market_pubkey,
            signature,
        } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            let events = sdk.parse_events_from_transaction(&signature).await.unwrap();
            log_market_events(&sdk, events);
            Ok(())
        }
        Command::GetMarketStatus { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            // Get market account
            let mut market_account_data = sdk.client.get_account_data(&market_pubkey).await?;
            let (header_bytes, _) = market_account_data.split_at_mut(size_of::<MarketHeader>());
            let header = MarketHeader::try_from_slice(header_bytes)?;

            let status = MarketStatus::from(header.status);
            println!("Market status: {}", status);
            Ok(())
        }
    }
}


async fn create_airdrop_spl_ixs(
    sdk_client: &SDKClient,
    recipient_pubkey: &Pubkey,
    base_amount: u64,
    quote_amount: u64,
) -> anyhow::Result<Vec<Instruction>> {
    // Get base and quote mints from market metadata
    let market_metadata = sdk_client.get_active_market_metadata();
    let base_mint = market_metadata.base_mint;
    let quote_mint = market_metadata.quote_mint;

    let base_mint_account = Mint::unpack(&sdk_client.client.get_account_data(&base_mint).await?)?;

    let quote_mint_account = Mint::unpack(&sdk_client.client.get_account_data(&quote_mint).await?)?;

    let quote_mint_authority = quote_mint_account.mint_authority.unwrap();
    let base_mint_authority = base_mint_account.mint_authority.unwrap();

    if sdk_client
        .client
        .get_account(&quote_mint_authority)
        .await?
        .owner
        != devnet_token_faucet::id()
    {
        return Err(anyhow::anyhow!(
            "Quote mint authority is not owned by devnet-token-faucet"
        ));
    }

    if sdk_client
        .client
        .get_account(&base_mint_authority)
        .await?
        .owner
        != devnet_token_faucet::id()
    {
        return Err(anyhow::anyhow!(
            "Base mint authority is not owned by devnet-token-faucet"
        ));
    }

    // Get or create the ATA for the recipient. If doesn't exist, create token account
    let mut instructions = vec![];

    let recipient_ata_base =
        spl_associated_token_account::get_associated_token_address(recipient_pubkey, &base_mint);

    if sdk_client
        .client
        .get_account(&recipient_ata_base)
        .await
        .is_err()
    {
        println!("Error retrieving ATA. Creating ATA");
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account(
                &sdk_client.client.payer.pubkey(),
                recipient_pubkey,
                &base_mint,
                &spl_token::id(),
            ),
        )
    };

    let recipient_ata_quote =
        spl_associated_token_account::get_associated_token_address(recipient_pubkey, &quote_mint);

    if sdk_client
        .client
        .get_account(&recipient_ata_quote)
        .await
        .is_err()
    {
        println!("Error retrieving ATA. Creating ATA");
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account(
                &sdk_client.client.payer.pubkey(),
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

    Ok(instructions)
}

fn log_market_events(sdk: &SDKClient, market_events: Vec<PhoenixEvent>) {
    for event in market_events {
        match event.details {
            MarketEventDetails::Fill(fill) => {
                if event.market != sdk.active_market_key {
                    continue;
                }
                let Fill {
                    maker,
                    taker,
                    price_in_ticks,
                    base_lots_filled,
                    side_filled,
                    ..
                } = fill;
                let keys = vec![];
                let fill_data = vec![
                    maker.to_string(),
                    taker.to_string(),
                    (sdk.ticks_to_float_price(price_in_ticks)).to_string(),
                    format!("{:?}", side_filled),
                    get_decimal_string(
                        sdk.base_lots_to_base_amount(base_lots_filled),
                        sdk.base_decimals,
                    ),
                ];
                println!("{}", finalize_log(keys, fill_data));
            }
            MarketEventDetails::Place(place) => {
                if event.market != sdk.active_market_key {
                    continue;
                }
                let Place {
                    order_sequence_number,
                    client_order_id: _,
                    maker,
                    price_in_ticks,
                    base_lots_placed,
                } = place;
                let side = Side::from_order_sequence_number(order_sequence_number);
                let keys = initialize_log(&event, "Place".to_string());
                let place_data = vec![
                    maker.to_string(),
                    "".to_string(),
                    (sdk.ticks_to_float_price(price_in_ticks)).to_string(),
                    format!("{:?}", side),
                    get_decimal_string(
                        sdk.base_lots_to_base_amount(base_lots_placed),
                        sdk.base_decimals,
                    ),
                ];

                println!("{}", finalize_log(keys, place_data));
            }
            MarketEventDetails::Reduce(reduce) => {
                if event.market != sdk.active_market_key {
                    continue;
                }
                let Reduce {
                    order_sequence_number,
                    maker,
                    price_in_ticks,
                    base_lots_removed,
                    ..
                } = reduce;
                let side = Side::from_order_sequence_number(order_sequence_number);
                let keys = initialize_log(&event, "Reduce".to_string());

                let reduce_data = vec![
                    maker.to_string(),
                    "".to_string(),
                    (sdk.ticks_to_float_price(price_in_ticks)).to_string(),
                    format!("{:?}", side),
                    get_decimal_string(
                        sdk.base_lots_to_base_amount(base_lots_removed),
                        sdk.base_decimals,
                    ),
                ];
                println!("{}", finalize_log(keys, reduce_data));
            }
            _ => {
                continue;
            }
        }
    }
}
fn initialize_log(event: &PhoenixEvent, event_type: String) -> Vec<String> {
    let base_schema: Vec<String> = vec![
        "market".to_string(),
        "event_type".to_string(),
        "timestamp".to_string(),
        "signature".to_string(),
        "slot".to_string(),
        "sequence_number".to_string(),
        "event_index".to_string(),
    ];
    let base = vec![
        event.market.to_string(),
        event_type,
        event.timestamp.to_string(),
        event.signature.to_string(),
        event.slot.to_string(),
        event.sequence_number.to_string(),
        event.event_index.to_string(),
    ];
    base_schema
        .iter()
        .zip(base.iter())
        .map(|(a, b)| format!("{}: {}", a, b))
        .collect::<Vec<String>>()
}

fn finalize_log(mut log: Vec<String>, data: Vec<String>) -> String {
    let event_schema: Vec<String> = vec![
        "maker".to_string(),
        "taker".to_string(),
        "price".to_string(),
        "side".to_string(),
        "quantity".to_string(),
    ];
    log.extend_from_slice(
        &event_schema
            .iter()
            .zip(data.iter())
            .map(|(a, b)| format!("{}: {}", a, b))
            .collect::<Vec<String>>(),
    );
    log.join(", ")
}

mod command;
mod processor;
mod helpers;
use crate::command::Command;
use crate::processor::*;

use anyhow::anyhow;
use clap::Parser;
use ellipsis_client::EllipsisClient;
use phoenix_sdk::sdk_client::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signer::{
    keypair::{read_keypair_file, Keypair},
    Signer,
};
use std::env;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[clap(short, long, default_value = "dev")]
    network: String,
    #[clap(subcommand)]
    command: Command,
}

pub fn get_network(network_str: &str) -> &str {
    match network_str {
        "devnet" | "dev" => "https://api.devnet.solana.com",
        "mainnet" | "main" | "mainnet-beta" => "https://api.mainnet-beta.solana.com",
        _ => network_str,
    }
}

pub fn is_devnet(network_str: &str) -> bool {
    matches!(network_str, "devnet" | "dev")
}

pub fn get_payer_keypair() -> Keypair {
    match env::var("PAYER").is_ok() {
        true => Keypair::from_base58_string(&env::var("PAYER").expect("$PAYER is not set")[..]),
        false => read_keypair_file(&*shellexpand::tilde("~/.config/solana/id.json"))
            .map_err(|e| anyhow!(e.to_string()))
            .unwrap(),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let network_url = get_network(&cli.network);

    let payer = get_payer_keypair();

    println!("Current payer: {}", payer.pubkey());
    println!("Current network: {}", network_url);

    let client = EllipsisClient::from_rpc(
        RpcClient::new_with_commitment(network_url, CommitmentConfig::confirmed()),
        &payer,
    )?;

    match cli.command {
        Command::GetMarket { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_market(&market_pubkey, &sdk).await
        }
        Command::GetAllMarkets => process_get_all_markets(&client),
        Command::GetTradersForMarket { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_traders_for_market(&market_pubkey, &sdk).await
        }
        Command::GetTopOfBook { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_top_of_book(&market_pubkey, &sdk).await
        }
        Command::GetBookLevels {
            market_pubkey,
            levels,
        } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_book(&market_pubkey, &sdk, levels).await
        }
        Command::GetFullBook { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_full_book(&market_pubkey, &sdk).await
        }
        Command::GetTransaction {
            market_pubkey,
            signature,
        } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_transaction(&signature, &sdk).await
        }
        Command::GetMarketStatus { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_market_status(&market_pubkey, &sdk).await
        }
        Command::GetSeatInfo {
            market_pubkey,
            trader_pubkey,
        } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_seat_info(&market_pubkey, &trader_pubkey, &sdk).await
        }
        Command::GetOpenOrders {
            market_pubkey,
            trader_pubkey,
        } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_open_orders(&market_pubkey, &trader_pubkey, &sdk).await
        }
        Command::MintTokens {
            mint_ticker,
            recipient_pubkey,
            amount,
        } => {
            if !is_devnet(&network_url) {
                println!("Command only valid for devnet");
                return Ok(());
            }
            process_mint_tokens(&client, &payer, &recipient_pubkey, mint_ticker, amount).await
        }
        Command::MintTokensForMarket {
            market_pubkey,
            recipient_pubkey,
            base_amount,
            quote_amount,
        } => {
            if !is_devnet(&network_url) {
                println!("Command only valid for devnet");
                return Ok(());
            }
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_mint_tokens_for_market(
                &sdk,
                &recipient_pubkey,
                base_amount,
                quote_amount,
            )
            .await
        }
    }
}

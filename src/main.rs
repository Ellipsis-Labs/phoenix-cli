mod command;
mod helpers;
mod processor;
use crate::command::PhoenixCLICommand;
use crate::processor::{
    process_get_all_markets::*, process_get_book_levels::*, process_get_full_book::*,
    process_get_market::*, process_get_market_status::*, process_get_open_orders::*,
    process_get_seat_info::*, process_get_top_of_book::*, process_get_traders_for_market::*,
    process_get_transaction::*, process_mint_tokens::*, process_mint_tokens_for_market::*,
    process_request_seat::*,
};
use anyhow::anyhow;
use clap::Parser;
use ellipsis_client::EllipsisClient;
use phoenix_sdk::sdk_client::*;
use solana_cli_config::{Config, ConfigInput, CONFIG_FILE};
use solana_client::rpc_client::RpcClient;
use solana_sdk::signer::keypair::{read_keypair_file, Keypair};
use solana_sdk::signer::Signer;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[clap(subcommand)]
    command: PhoenixCLICommand,
    #[clap(global = true, short, long)]
    url: Option<String>,
    #[clap(global = true, short, long)]
    keypair_path: Option<String>,
    #[clap(global = true, short, long)]
    commitment: Option<String>,
}

pub fn get_network(network_str: &str) -> &str {
    match network_str {
        "devnet" | "dev" | "d" => "https://api.devnet.solana.com",
        "mainnet" | "main" | "m" | "mainnet-beta" => "https://api.mainnet-beta.solana.com",
        "localnet" | "localhost" | "l" | "local" => "http://localhost:8899",
        _ => network_str,
    }
}

pub fn get_payer_keypair_from_path(path: &str) -> Keypair {
    read_keypair_file(&*shellexpand::tilde(path))
        .map_err(|e| anyhow!(e.to_string()))
        .unwrap()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let config = match CONFIG_FILE.as_ref() {
        Some(config_file) => Config::load(config_file).unwrap_or_else(|_| {
            println!("Failed to load config file: {}", config_file);
            Config::default()
        }),
        None => Config::default(),
    };
    // let config = Config::load(&CONFIG_FILE.as_ref().unwrap()).expect("Failed to load config file");
    let commitment =
        ConfigInput::compute_commitment_config("", &cli.commitment.unwrap_or(config.commitment)).1;
    let payer = get_payer_keypair_from_path(&cli.keypair_path.unwrap_or(config.keypair_path));
    let network_url = &get_network(&cli.url.unwrap_or(config.json_rpc_url)).to_string();
    let client = EllipsisClient::from_rpc(
        RpcClient::new_with_commitment(network_url, commitment),
        &payer,
    )?;

    match cli.command {
        PhoenixCLICommand::GetMarket { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_market(&market_pubkey, &sdk).await
        }
        PhoenixCLICommand::GetAllMarkets => process_get_all_markets(&client),
        PhoenixCLICommand::GetTradersForMarket { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_traders_for_market(&market_pubkey, &sdk).await
        }
        PhoenixCLICommand::GetTopOfBook { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_top_of_book(&market_pubkey, &sdk).await
        }
        PhoenixCLICommand::GetBookLevels {
            market_pubkey,
            levels,
        } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_book_levels(&market_pubkey, &sdk, levels).await
        }
        PhoenixCLICommand::GetFullBook { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_full_book(&market_pubkey, &sdk).await
        }
        PhoenixCLICommand::GetTransaction {
            market_pubkey,
            signature,
        } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_transaction(&signature, &sdk).await
        }
        PhoenixCLICommand::GetMarketStatus { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_market_status(&market_pubkey, &sdk).await
        }
        PhoenixCLICommand::GetSeatInfo {
            market_pubkey,
            trader_pubkey,
        } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_seat_info(
                &market_pubkey,
                &trader_pubkey.unwrap_or_else(|| payer.pubkey()),
                &sdk,
            )
            .await
        }
        PhoenixCLICommand::GetOpenOrders {
            market_pubkey,
            trader_pubkey,
        } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_get_open_orders(
                &market_pubkey,
                &trader_pubkey.unwrap_or_else(|| payer.pubkey()),
                &sdk,
            )
            .await
        }
        PhoenixCLICommand::RequestSeat { market_pubkey } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_request_seat(&market_pubkey, &sdk).await
        }
        PhoenixCLICommand::MintTokens {
            mint_ticker,
            recipient_pubkey,
            amount,
        } => process_mint_tokens(&client, &payer, &recipient_pubkey, mint_ticker, amount).await,
        PhoenixCLICommand::MintTokensForMarket {
            market_pubkey,
            recipient_pubkey,
            base_amount,
            quote_amount,
        } => {
            let sdk = SDKClient::new(&market_pubkey, &payer, network_url).await;
            process_mint_tokens_for_market(&sdk, &recipient_pubkey, base_amount, quote_amount).await
        }
    }
}

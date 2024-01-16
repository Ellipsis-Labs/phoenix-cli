mod command;

use crate::command::PhoenixCLICommand;
use anyhow::anyhow;
use clap::Parser;
use ellipsis_client::EllipsisClient;
use phoenix_cli_processor::processor::process_claim_seat::process_claim_seat;
use phoenix_cli_processor::processor::process_evict_seat::process_evict_seat;
use phoenix_cli_processor::processor::{
    process_get_all_markets::*, process_get_book_levels::*, process_get_full_book::*,
    process_get_market::*, process_get_market_status::*, process_get_open_orders::*,
    process_get_seat_info::*, process_get_seat_manager_info::*, process_get_top_of_book::*,
    process_get_traders_for_market::*, process_get_transaction::*, process_mint_tokens::*,
    process_mint_tokens_for_market::*, process_request_seat::*,
};
use phoenix_sdk::sdk_client::*;
use solana_cli_config::{Config, ConfigInput, CONFIG_FILE};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signer::keypair::{read_keypair_file, Keypair};
use solana_sdk::signer::Signer;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[clap(subcommand)]
    command: PhoenixCLICommand,
    /// Optionally include your RPC endpoint. Use "local", "dev", "main" for default endpoints. Defaults to your Solana CLI config file.
    #[clap(global = true, short, long)]
    url: Option<String>,
    /// Optionally include your keypair path. Defaults to your Solana CLI config file.
    #[clap(global = true, short, long)]
    keypair_path: Option<String>,
    /// Optionally include a commitment level. Defaults to your Solana CLI config file.
    #[clap(global = true, short, long)]
    commitment: Option<String>,
    /// Optionally include a priority fee, in how many micro lamports you want to pay per compute unit. Defaults to no priority fee. Max is 1 million
    #[clap(global = true, short, long)]
    prio_fee: Option<u64>,
    /// Optionally include the number of compute units you want to pay for. Only works if a priority fee is also set. If prio fee is set, this defaults to 1.4 million
    #[clap(global = true, short, long)]
    num_compute_units: Option<u32>,
}

pub fn get_network(network_str: &str) -> &str {
    match network_str {
        "devnet" | "dev" | "d" => "https://api.devnet.solana.com",
        "mainnet" | "main" | "m" | "mainnet-beta" => "https://api.mainnet-beta.solana.com",
        "localnet" | "localhost" | "l" | "local" => "http://localhost:8899",
        _ => network_str,
    }
}

pub fn get_payer_keypair_from_path(path: &str) -> anyhow::Result<Keypair> {
    read_keypair_file(&*shellexpand::tilde(path)).map_err(|e| anyhow!(e.to_string()))
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
    let commitment =
        ConfigInput::compute_commitment_config("", &cli.commitment.unwrap_or(config.commitment)).1;
    let payer = get_payer_keypair_from_path(&cli.keypair_path.unwrap_or(config.keypair_path))
        .expect("Keypair file does not exist. Please run `solana-keygen new`");
    let network_url = &get_network(&cli.url.unwrap_or(config.json_rpc_url)).to_string();
    let client = EllipsisClient::from_rpc(
        RpcClient::new_with_commitment(network_url.to_string(), commitment),
        &payer,
    )?;

    let mut sdk = SDKClient::new(&payer, network_url).await?;

    let prio_fee_instructions = match cli.prio_fee {
        Some(compute_unit_price) => {
            let num_compute_units = cli.num_compute_units.unwrap_or(1_200_000);
            assert!(
                num_compute_units <= 1_400_000,
                "Number of compute units must be less than 1.4 million"
            );
            assert!(
                compute_unit_price <= 1_000_000,
                "Priority fee must be less than 1 million micro lamports"
            );
            let cu_limit = ComputeBudgetInstruction::set_compute_unit_limit(num_compute_units);
            let cu_price = ComputeBudgetInstruction::set_compute_unit_price(compute_unit_price);
            vec![cu_limit, cu_price]
        }
        None => vec![],
    };

    match cli.command {
        PhoenixCLICommand::GetMarket { market_pubkey } => {
            sdk.add_market(&market_pubkey).await?;
            process_get_market(&market_pubkey, &sdk).await?
        }
        PhoenixCLICommand::GetAllMarkets { no_gpa } => {
            if no_gpa {
                process_get_all_markets_no_gpa(&client, network_url).await?
            } else {
                process_get_all_markets(&client).await?
            }
        }
        PhoenixCLICommand::GetTradersForMarket { market_pubkey } => {
            sdk.add_market(&market_pubkey).await?;
            process_get_traders_for_market(&market_pubkey, &sdk).await?
        }
        PhoenixCLICommand::GetTopOfBook { market_pubkey } => {
            sdk.add_market(&market_pubkey).await?;
            process_get_top_of_book(&market_pubkey, &sdk).await?
        }
        PhoenixCLICommand::GetBookLevels {
            market_pubkey,
            levels,
        } => {
            sdk.add_market(&market_pubkey).await?;
            process_get_book_levels(&market_pubkey, &sdk, levels).await?
        }
        PhoenixCLICommand::GetFullBook { market_pubkey } => {
            sdk.add_market(&market_pubkey).await?;
            process_get_full_book(&market_pubkey, &sdk).await?
        }
        PhoenixCLICommand::GetTransaction { signature } => {
            process_get_transaction(&signature, &mut sdk).await?
        }
        PhoenixCLICommand::GetMarketStatus { market_pubkey } => {
            sdk.add_market(&market_pubkey).await?;
            process_get_market_status(&market_pubkey, &sdk).await?
        }
        PhoenixCLICommand::GetSeatInfo {
            market_pubkey,
            trader_pubkey,
        } => {
            sdk.add_market(&market_pubkey).await?;
            process_get_seat_info(
                &market_pubkey,
                &trader_pubkey.unwrap_or_else(|| payer.pubkey()),
                &sdk,
            )
            .await?
        }
        PhoenixCLICommand::GetOpenOrders {
            market_pubkey,
            trader_pubkey,
        } => {
            sdk.add_market(&market_pubkey).await?;
            process_get_open_orders(
                &market_pubkey,
                &trader_pubkey.unwrap_or_else(|| payer.pubkey()),
                &sdk,
            )
            .await?
        }
        PhoenixCLICommand::RequestSeat { market_pubkey } => {
            sdk.add_market(&market_pubkey).await?;
            process_request_seat(&market_pubkey, &sdk, prio_fee_instructions).await?
        }
        PhoenixCLICommand::MintTokens {
            mint_ticker,
            recipient_pubkey,
            amount,
        } => process_mint_tokens(&client, &payer, &recipient_pubkey, mint_ticker, amount).await?,
        PhoenixCLICommand::MintTokensForMarket {
            market_pubkey,
            recipient_pubkey,
            base_amount,
            quote_amount,
        } => {
            sdk.add_market(&market_pubkey).await?;
            process_mint_tokens_for_market(
                &sdk,
                &market_pubkey,
                &recipient_pubkey,
                base_amount,
                quote_amount,
            )
            .await?
        }
        PhoenixCLICommand::GetSeatManagerInfo { market_pubkey } => {
            sdk.add_market(&market_pubkey).await?;
            process_get_seat_manager_info(&sdk.client, &market_pubkey).await?;
        }
        PhoenixCLICommand::ClaimSeat { market_pubkey } => {
            sdk.add_market(&market_pubkey).await?;
            process_claim_seat(&sdk.client, &market_pubkey, prio_fee_instructions).await?
        }
        PhoenixCLICommand::EvictSeat {
            market_pubkey,
            trader_to_evict,
        } => {
            sdk.add_market(&market_pubkey).await?;
            process_evict_seat(
                &sdk.client,
                &market_pubkey,
                &trader_to_evict,
                prio_fee_instructions,
            )
            .await?
        }
    }

    Ok(())
}

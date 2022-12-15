use clap::Parser;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;

// #[clap(author, version, about)]
#[derive(Debug, Clone, Parser)]
pub enum Command {
    /// Get summary information on all markets
    GetAllMarkets,
    /// Get detailed information on a specific market
    GetMarket {
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
    },
    /// Get active traders for a given market
    GetTradersForMarket {
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
    },
    /// Get the best bid and ask price for a given market
    GetTopOfBook {
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
    },
    /// Get the first N levels of the order book for a given market.
    /// Default is 10 levels
    GetBookLevels {
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
        #[clap(short, long, required = false, default_value = "10")]
        levels: u64,
    },
    /// Get the full order book for a given market
    GetFullBook {
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
    },
    /// Get the market events that occured in a given transaction signature
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
    /// Get the status and address of a seat for a given market and trader
    GetSeatInfo {
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
        #[clap(short, long, required = true)]
        trader_pubkey: Pubkey,
    },
    /// Get all open orders on a given market for a trader
    GetOpenOrders {
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
        #[clap(short, long, required = true)]
        trader_pubkey: Pubkey,
    },
    RequestSeat { 
        #[clap(short, long, required = true)]
        market_pubkey: Pubkey,
        #[clap(short, long, required = true)]
        trader_pubkey: Pubkey,
    },
    /// Mint tokens to a recipient for a given ticker string. Default amount is 100_000_000_000.
    /// Devnet only
    MintTokens {
        #[clap(short, long, required = true)]
        mint_ticker: String,
        #[clap(short, long, required = true)]
        recipient_pubkey: Pubkey,
        #[clap(short, long, required = false, default_value = "100000000000")]
        amount: u64,
    },
    /// Mint both base and quote tokens to a recipient for a given market. Default amounts are 100_000_000_000 for base and 100_000_000 for quote.
    /// Devnet only
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
}

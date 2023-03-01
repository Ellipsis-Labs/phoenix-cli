use clap::Parser;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;

// #[clap(author, version, about)]
#[derive(Debug, Clone, Parser)]
pub enum PhoenixCLICommand {
    /// Get summary information on all markets
    GetAllMarkets {
        /// Optionally skip the GetProgramAccounts network call. This will read a static list of markets in the config file instead.
        /// Only for mainnet and highly recommended for mainnet
        #[clap(short, long, required = false)]
        no_gpa: bool,
    },
    /// Get detailed information on a specific market
    GetMarket { market_pubkey: Pubkey },
    /// Get active traders for a given market
    GetTradersForMarket { market_pubkey: Pubkey },
    /// Get the best bid and ask price for a given market
    GetTopOfBook { market_pubkey: Pubkey },
    /// Get the first N levels of the order book for a given market.
    /// Default is 10 levels
    GetBookLevels {
        market_pubkey: Pubkey,
        #[clap(short, long, required = false, default_value = "10")]
        levels: u64,
    },
    /// Get the full order book for a given market
    GetFullBook { market_pubkey: Pubkey },
    /// Get the market events that occured in a given transaction signature
    GetTransaction {
        signature: Signature,
    },
    /// Get the current status of a market
    GetMarketStatus { market_pubkey: Pubkey },
    /// Get the status and address of a seat for a given market and trader
    GetSeatInfo {
        market_pubkey: Pubkey,
        /// Pubkey of the trader associated with the seat. Defaults to the current payer
        #[clap(short, long, required = false)]
        trader_pubkey: Option<Pubkey>,
    },
    /// Get all open orders on a given market for a trader
    GetOpenOrders {
        market_pubkey: Pubkey,
        /// Pubkey of the trader for whom to get open orders. Defaults to the current payer
        #[clap(short, long, required = false)]
        trader_pubkey: Option<Pubkey>,
    },
    /// Send a transaction on chain to allocate a seat for the payer on the given market. This will cost ~.0018 SOL for rent. 
    /// Note that the seat will have to then be approved by the market authority.
    RequestSeat { market_pubkey: Pubkey },
    /// Mint tokens to a recipient for a given ticker string (for example SOL or USDC). Default amount is 100_000_000_000.
    /// This is only for markets associated with the ellipsis token faucet. 
    MintTokens {
        /// Ticker string, example: SOL
        mint_ticker: String,
        /// Pubkey of the recipient of the tokens
        recipient_pubkey: Pubkey,
        /// Amount in atoms (1 * 10*(-decimals))
        #[clap(short, long, required = false, default_value = "100000000000")]
        amount: u64,
    },
    /// Mint both base and quote tokens to a recipient for a given market. Default amounts are 100_000_000_000 for base and 100_000_000 for quote.
    /// This is only for markets associated with the ellipsis token faucet. 
    MintTokensForMarket {
        market_pubkey: Pubkey,
        /// Pubkey of the recipient of the tokens
        recipient_pubkey: Pubkey,
        /// Amount in atoms (1 * 10*(-decimals))
        #[clap(short, long, required = false, default_value = "100000000000")]
        base_amount: u64,
        /// Amount in atoms (1 * 10*(-decimals))
        #[clap(short, long, required = false, default_value = "100000000")]
        quote_amount: u64,
    },
}

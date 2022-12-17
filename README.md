# phoenix-cli
CLI for interacting with the Phoenix DEX

# Running the CLI
TODO: include how to run the binary, how to see all commands in terminal (phoenix-cli --help), how to see specific help (phoenix-cli <COMMAND> --help)

# Commands
TODO: improve docs, include succint info about input parameters and return values 
get-all-markets
Returns summary information on all markets that exist on Phoenix. Summary information includes market key, base and quote token keys, and authority key. 

### get-market
Returns detailed information on a specific market. Information includes market balance's of the base and quote tokens, base and quote token keys, base lot size, quote lot size, tick size, and taker fees in basis points. 

### get-traders-for-market
Returns all trader keys that have an approved seat on a given market.

### get-top-of-book
Returns the best bid and best ask on a given market. 

### get-book-levels
Returns the top N levels of a market's orderbook. N is by default set to 10. 

### get-full-book
Returns the full orderbook for a given market.

### get-transaction
Returns a summary of the market events that occured (Place, Fill, Reduce/Cancel) in a given transaction signature for the given market. 

### get-market-status
Returns the status of a given market. Markets can be in the following states: Active, PostOnly, Paused, Closed, Uninitialized, Tombstoned.

### get-seat-info
Returns the status and address of a trader's seat. By default, returns the payer's seat info. Seats can be in the following states: Approved, NotApproved, Retired

### get-open-orders
Returns all open orders on a given market for a trader. By default, returns the payer's open orders. Returns the side, orderID, price in ticks, price, and size for each order. 

### request-seat
Requests a seat for the payer on the given market. Note that the seat will have to then be approved by the market authority in order to place limit orders. 

### mint-tokens
Mints tokens of the ticker_string (example: SOL) to the given pubkey. Default amount is 100_000_000_000. 

### mint-tokens-for-market
Mints the base and quote tokens of the given market to the given pubkey. Default amounts are 100_000_000_000 for base and 100_000_000 for quote.











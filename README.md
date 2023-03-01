# phoenix-cli
CLI for interacting with the Phoenix DEX

## Program Deployments

| Program     | Devnet                                         | 
| ----------- | ---------------------------------------------- |
| Phoenix Dex | `PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY`  |

| Program     | Mainnet                                        | 
| ----------- | ---------------------------------------------- |
| Phoenix Dex | `PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY`  |

## Installation 

For Linux and MacOS, run the install script from your terminal
```
bash <(curl -sSf https://raw.githubusercontent.com/Ellipsis-Labs/phoenix-cli/master/phoenix-cli-install.sh)
```

Test the installation of cli by running
```
phoenix-cli --help
```

## Running the CLI

To view a list of all available commands, run `phoenix-cli --help`
<img width="1072" alt="image" src="https://user-images.githubusercontent.com/9097655/208983165-b5472419-f006-4e64-904e-e34f42bac6cc.png">


To zoom in on a specific command, run `phoenix-cli <COMMAND> --help`

Optionally include the following parameters when running the cli: 
* `-u, --url` Include your RPC endpoint. Use "local", "dev", and "main" for the respective default endpoints. Defaults to your Solana CLI config settings - if the config isn't found, defaults to mainnet. 
* `-k, --keypair-path` Include the path to the keypair you wish to use. Defaults to your Solana CLI config settings - if the config isn't found, defaults to `.config/solana/id.json`
* `-c, --commitment` Include a commitment level for the RPC. Defaults to your Solana CLI config settings - if the config isn't found, defaults to Confirmed

## Commands


### get-all-markets
Returns summary information on all markets that exist on Phoenix. Summary information includes market key, base and quote token keys, and authority key.

`$ phoenix-cli -u main get-all-markets --no-gpa`
```
Found 2 market(s)
--------------------------------------------
Market: 14CAwu3LiBBk5fcHGdTsFyVxDwvpgFiSfDwgPJxECcE5
Base Token: 7Z6Kczxo8ViRpfnsVvVaATB5fQ8bN2CQpxP8DHfd1vz5
Quote Token: 5zUmtDCDeR17UYjvKKqvYp3S9pqcZA69cDoYPtojseJ4
Authority: 9odqiJyK4zCMNfPi6AUE6gi9tomqZKPFYcDiokMXYRzS
--------------------------------------------
Market: 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg
Base Token: So11111111111111111111111111111111111111112
Quote Token: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
Authority: 9odqiJyK4zCMNfPi6AUE6gi9tomqZKPFYcDiokMXYRzS
```

### get-market
Returns detailed information on a specific market. Information includes market balance's of the base and quote tokens, base and quote token keys, base lot size, quote lot size, tick size, and taker fees in basis points. 

`$ phoenix-cli -u main get-market -m 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg`
```
Market: 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg
Status: Active
Authority: 9odqiJyK4zCMNfPi6AUE6gi9tomqZKPFYcDiokMXYRzS
Sequence number: 696709
Base Vault balance: 0.000
Quote Vault balance: 10.485
Base Token: So11111111111111111111111111111111111111112
Quote Token: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
Base vault key: 8g4Z9d6PqGkgH31tMW6FwxGhwYJrXpxZHQrkikpLJKrG
Quote vault key: 3HSYXeGc3LjEPCuzoNDjQN37F1ebsSiR4CqXVqQCdekZ
Base Lot Size, in whole units: 0.001
Quote Lot Size, in whole units: 0.000001
Tick size in quote atoms per base unit: 0.001
Taker fees in basis points: 2
Fee destination pubkey: 6pwvUFHxtwNrcMqb12V3ni2FXcMnvTWvBWX5DXmPpg1Y
Raw base units per base unit: 1
Market Size Params: MarketSizeParams { bids_size: 4096, asks_size: 4096, num_seats: 8321 }
Successor pubkey: 9odqiJyK4zCMNfPi6AUE6gi9tomqZKPFYcDiokMXYRzS
Uncollected fees, in quote units: 10.48482
Collected fees, in quote units: 0.0
```

### get-traders-for-market
Returns all trader keys that have an approved seat on a given market.

`$ phoenix-cli -u main get-traders-for-market 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg`
```
Found 3 trader(s). Printing traders with locked or free lots
```

### get-top-of-book
Returns the best bid and best ask on a given market. 

`$ phoenix-cli -u main get-top-of-book 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg`
```
       22.990  5.838
 5.843 22.980 
 ```
 
### get-book-levels
Returns the top N levels of a market's orderbook. N is by default set to 10. 

`$ phoenix-cli -u main get-book-levels 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg -l 5`
```
          23.030  109.725
          23.015   66.583
          23.005   29.987
          22.995   15.006
          22.990    4.838
   5.843  22.980         
  15.031  22.975         
  30.087  22.965         
  66.917  22.955         
 110.552  22.940  
```
### get-full-book
Returns the full orderbook for a given market.

`$ phoenix-cli -u main get-full-book 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg`
```
          23.210  409.500
          23.025  166.320
          23.005  109.954
          22.990   96.747
          22.980   15.025
          22.975    5.845
   5.850  22.965         
  15.050  22.960         
  30.125  22.950         
  67.057  22.930         
 110.784  22.915         
 168.137  22.895         
 426.985  22.710         
```

### get-transaction
Returns a summary of the market events that occured (Place, Fill, Reduce/Cancel) in a given transaction signature for the given market. 

`$ phoenix-cli -u main get-transaction 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg 4gw6UDWsDCWrh2eqYxvVzbVyywfPVo24V2qMTSVGJJAdxvv9Tx4pBrqE1cLTgomP2QkZ7wigbjoN3GpibhJY8PFV`
```

```

### get-market-status
Returns the status of a given market. Markets can be in the following states: Active, PostOnly, Paused, Closed, Uninitialized, Tombstoned.

`$ phoenix-cli -u main get-market-status 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg`
```
Market status: Active
```

### get-seat-info
Returns the status and address of a trader's seat. By default, returns the payer's seat info. Seats can be in the following states: Approved, NotApproved, Retired

`$ phoenix-cli -u main get-seat-info -m 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg -t 3HBWHuyxWv4uN8U8SeukocrWPfLZJqrtj9DgDHsGo2HR`
```
Seat address: GGyZqgoqnKsvMTsmSSkTrDjtdSFUsEoioKz9Yr2vEnZa
Seat status: Approved
```

### get-open-orders
Returns all open orders on a given market for a trader. By default, returns the payer's open orders. Returns the side, orderID, price in ticks, price, and size for each order. 

`$ phoenix-cli -u main get-open-orders 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg -t mkrc4jMLEPRoKLUnNL7Ctnwb7uJykbwiYvFjB4sw9Z9`
```
Open Bids
ID                   | Price (ticks)        | Price      | Quantity  
18446744073708399662 | 2385                 | 22.925     | 5.870     
18446744073708399660 | 2384                 | 22.920     | 15.100    
18446744073708399658 | 2382                 | 22.910     | 30.226    
18446744073708399656 | 2380                 | 22.900     | 67.226    
18446744073708399654 | 2377                 | 22.885     | 111.064   

Open Asks
ID                   | Price (ticks)        | Price      | Quantity  
1151952              | 2387                 | 22.935     | 4.865     
1151954              | 2388                 | 22.940     | 15.075    
1151956              | 2390                 | 22.950     | 30.125    
1151958              | 2392                 | 22.960     | 66.889    
1151960              | 2395                 | 22.975     | 110.229   
1151962              | 2399                 | 22.995     | 166.736   
```

### request-seat
Requests a seat for the payer on the given market. Note that the seat will have to then be approved by the market authority in order to place limit orders. 

`$ phoenix-cli -u main request-seat 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg`
```
Requested seat, transaction signature: 3Qq7MZQ8XoLeT8fSfeFBTxRy8zFPvCFPbvwU2Zhu16gKT3o8tHo8HRxvHfyb75dvuJjDqo3sTpvfGL9v3tco8nAN
```

### mint-tokens
Mints tokens of the ticker_string (example: SOL) to the given pubkey. Default amount is 100_000_000_000. 

`$ phoenix-cli -u main mint-tokens -m SOL -a 100000 -r aChXgDyJn7g5BCkjccisGc78LrQZKEmNgt5sz8Tdkzn`
```
Creating ATA
100000 Tokens minted! Mint pubkey: B1sL3zxwyVnDGzRWCAsBkjL23wyu8HgwQP4XxgnHiSrv,  Recipient address: aChXgDyJn7g5BCkjccisGc78LrQZKEmNgt5sz8Tdkzn
```

### mint-tokens-for-market
Mints the base and quote tokens of the given market to the given pubkey. Default amounts are 100_000_000_000 for base and 100_000_000 for quote.

`$ phoenix-cli -u main mint-tokens-for-market 14CAwu3LiBBk5fcHGdTsFyVxDwvpgFiSfDwgPJxECcE5 -r aChXgDyJn7g5BCkjccisGc78LrQZKEmNgt5sz8Tdkzn`
```
Creating ATA for base token
Creating ATA for quote token
Tokens minted! Signature: 2mN6o7gBB41UFEboQuCMaeG1t5qQ1uRAvTDoXUhsk1yBoKXQtrXsHVtkQAT9R3oRUSPbhDkZjCQtNtjcYP4TqwVV
```












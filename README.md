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
Returns summary information on all markets that exist on Phoenix. Summary information includes market key, base and quote token keys, and authority key. Highly recommended to use the no-gpa flag for mainnet. 

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

`$ phoenix-cli -u main get-market 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg`
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
--------------------------------
Trader pubkey: 3HBWHuyxWv4uN8U8SeukocrWPfLZJqrtj9DgDHsGo2HR
Base token locked: 116.873
Base token free: 6.666
Quote token locked: 2647.022716
Quote token free: 1222.250847
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
Returns a summary of the market events that occured (Place, Fill, Reduce/Cancel) in a given transaction signature.

`$ phoenix-cli -u main get-transaction 4gw6UDWsDCWrh2eqYxvVzbVyywfPVo24V2qMTSVGJJAdxvv9Tx4pBrqE1cLTgomP2QkZ7wigbjoN3GpibhJY8PFV`
```
market: 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg, event_type: Fill, timestamp: 1677629539, signature: 4gw6UDWsDCWrh2eqYxvVzbVyywfPVo24V2qMTSVGJJAdxvv9Tx4pBrqE1cLTgomP2QkZ7wigbjoN3GpibhJY8PFV, slot: 180067446, sequence_number: 680904, event_index: 0, maker: 3HBWHuyxWv4uN8U8SeukocrWPfLZJqrtj9DgDHsGo2HR, taker: CcoiNhaTR88CSkEdsdeJpEMWnfCNqMf4HGGzXjwnvZF, price: 21.815, side: Bid, quantity: 2.288
market: 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg, event_type: Fill, timestamp: 1677629539, signature: 4gw6UDWsDCWrh2eqYxvVzbVyywfPVo24V2qMTSVGJJAdxvv9Tx4pBrqE1cLTgomP2QkZ7wigbjoN3GpibhJY8PFV, slot: 180067446, sequence_number: 680904, event_index: 1, maker: 3HBWHuyxWv4uN8U8SeukocrWPfLZJqrtj9DgDHsGo2HR, taker: CcoiNhaTR88CSkEdsdeJpEMWnfCNqMf4HGGzXjwnvZF, price: 21.811, side: Bid, quantity: 27.459
market: 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg, event_type: Fill, timestamp: 1677629539, signature: 4gw6UDWsDCWrh2eqYxvVzbVyywfPVo24V2qMTSVGJJAdxvv9Tx4pBrqE1cLTgomP2QkZ7wigbjoN3GpibhJY8PFV, slot: 180067446, sequence_number: 680904, event_index: 2, maker: 3HBWHuyxWv4uN8U8SeukocrWPfLZJqrtj9DgDHsGo2HR, taker: CcoiNhaTR88CSkEdsdeJpEMWnfCNqMf4HGGzXjwnvZF, price: 21.806, side: Bid, quantity: 17.066
Total quote token fees paid: 0.204193
```

### get-market-status
Returns the status of a given market. Markets can be in the following states: Active, PostOnly, Paused, Closed, Uninitialized, Tombstoned.

`$ phoenix-cli -u main get-market-status 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg`
```
Market status: Active
```

### get-seat-info
Returns the status and address of a trader's seat. By default, returns the payer's seat info. Seats can be in the following states: Approved, NotApproved, Retired

`$ phoenix-cli -u main get-seat-info 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg -t 3HBWHuyxWv4uN8U8SeukocrWPfLZJqrtj9DgDHsGo2HR`
```
Seat address: GGyZqgoqnKsvMTsmSSkTrDjtdSFUsEoioKz9Yr2vEnZa
Seat status: Approved
```

### get-open-orders
Returns all open orders on a given market for a trader. By default, returns the payer's open orders. Returns the side, orderID, price in ticks, price, and size for each order. 

`$ phoenix-cli -u main get-open-orders 14CAwu3LiBBk5fcHGdTsFyVxDwvpgFiSfDwgPJxECcE5 -t mkrc4jMLEPRoKLUnNL7Ctnwb7uJykbwiYvFjB4sw9Z9`
```
Open Bids
ID                   | Price (ticks)        | Price      | Quantity  
18446744073707873235 | 4466                 | 22.330     | 3.134     
18446744073707873233 | 4465                 | 22.325     | 8.062     
18446744073707873231 | 4462                 | 22.310     | 16.136    
18446744073707873237 | 4461                 | 22.305     | 35.866    
18446744073707873247 | 4457                 | 22.285     | 89.746    
18446744073707873229 | 4457                 | 22.285     | 59.232    
18446744073707873245 | 4420                 | 22.100     | 226.244   

Open Asks
ID                   | Price (ticks)        | Price      | Quantity  
1678379              | 4468                 | 22.340     | 3.133     
1678381              | 4469                 | 22.345     | 8.055     
1678383              | 4470                 | 22.350     | 16.107    
1678377              | 4473                 | 22.365     | 35.770    
1678385              | 4475                 | 22.375     | 58.994    
1678367              | 4483                 | 22.415     | 89.225    
1678369              | 4520                 | 22.600     | 221.238  
```

### request-seat
Send a transaction on chain to allocate a seat for the payer on the given market. This will cost ~.0018 SOL for rent. Note that the seat will have to then be approved by the market authority in order to place limit orders. 

`$ phoenix-cli -u main request-seat 4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg`
```
Requested seat, transaction signature: 3Qq7MZQ8XoLeT8fSfeFBTxRy8zFPvCFPbvwU2Zhu16gKT3o8tHo8HRxvHfyb75dvuJjDqo3sTpvfGL9v3tco8nAN
```

### mint-tokens
Mints tokens of the ticker_string (example: SOL) to the given pubkey. Default amount is 100_000_000_000. This command is only relevant for tokens associated with the ellipsis token faucet. On mainnet, this will only apply to the BASE/QUOTE market at address `14CAwu3LiBBk5fcHGdTsFyVxDwvpgFiSfDwgPJxECcE5`

`$ phoenix-cli -u main mint-tokens BASE aChXgDyJn7g5BCkjccisGc78LrQZKEmNgt5sz8Tdkzn -a 100000`
```
Creating ATA
100000 Tokens minted! Mint pubkey: 7Z6Kczxo8ViRpfnsVvVaATB5fQ8bN2CQpxP8DHfd1vz5,  Recipient address: aChXgDyJn7g5BCkjccisGc78LrQZKEmNgt5sz8Tdkzn
```

### mint-tokens-for-market
Mints the base and quote tokens of the given market to the given pubkey. Default amounts are 100_000_000_000 for base and 100_000_000 for quote. This command is only relevant for tokens associated with the ellipsis token faucet. On mainnet, this will only apply to the BASE/QUOTE market at address `14CAwu3LiBBk5fcHGdTsFyVxDwvpgFiSfDwgPJxECcE5`

`$ phoenix-cli -u main mint-tokens-for-market 14CAwu3LiBBk5fcHGdTsFyVxDwvpgFiSfDwgPJxECcE5 aChXgDyJn7g5BCkjccisGc78LrQZKEmNgt5sz8Tdkzn`
```
Creating ATA for base token
Creating ATA for quote token
Tokens minted! Signature: 2mN6o7gBB41UFEboQuCMaeG1t5qQ1uRAvTDoXUhsk1yBoKXQtrXsHVtkQAT9R3oRUSPbhDkZjCQtNtjcYP4TqwVV
```












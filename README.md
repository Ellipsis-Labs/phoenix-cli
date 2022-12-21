# phoenix-cli
CLI for interacting with the Phoenix DEX

## Program Deployments

| Program     | Devnet                                         | 
| ----------- | ---------------------------------------------- |
| Phoenix Dex | `phnxNHfGNVjpVVuHkceK3MgwZ1bW25ijfWACKhVFbBH`  |

## Installation 


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

`$ phoenix-cli -u dev get-all-markets`
```
Found 1 market(s)
--------------------------------------------
Market: 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8
Base Token: B1sL3zxwyVnDGzRWCAsBkjL23wyu8HgwQP4XxgnHiSrv
Quote Token: DK1gsSV2EubSE5S5FdXHpGzw2cAJNVzxeXRmAfxAMpU5
Authority: 2Hwmox2Qd84ZxPhKUGkTs7KUpjzYHWfHWbPT1kWvMf5b
```

### get-market
Returns detailed information on a specific market. Information includes market balance's of the base and quote tokens, base and quote token keys, base lot size, quote lot size, tick size, and taker fees in basis points. 

`$ phoenix-cli -u dev get-market -m 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8`
```
Base Vault balance: 1134.415
Quote Vault balance: 15417.129
Base Token: B1sL3zxwyVnDGzRWCAsBkjL23wyu8HgwQP4XxgnHiSrv
Quote Token: DK1gsSV2EubSE5S5FdXHpGzw2cAJNVzxeXRmAfxAMpU5
Base Lot Size: 0.001
Quote Lot Size: 0.000001
Tick size: 0.005
Taker fees in basis points: 5
```

### get-traders-for-market
Returns all trader keys that have an approved seat on a given market.

`$ phoenix-cli -u dev get-traders-for-market -m 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8`
```
--------------------------------
Trader pubkey: mkrc4jMLEPRoKLUnNL7Ctnwb7uJykbwiYvFjB4sw9Z9
Base token locked: 800.447
Base token free: 0.0
Quote token locked: 9729.96127
Quote token free: 11.995
--------------------------------
Trader pubkey: 2Hwmox2Qd84ZxPhKUGkTs7KUpjzYHWfHWbPT1kWvMf5b
Base token locked: 0.0
Base token free: 333.968
Quote token locked: 0.0
Quote token free: 4729.96854
```

### get-top-of-book
Returns the best bid and best ask on a given market. 

`$ phoenix-cli -u dev get-top-of-book -m 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8`
```
       11.990  5.838
 5.843 11.980 
 ```
 
### get-book-levels
Returns the top N levels of a market's orderbook. N is by default set to 10. 

`$ phoenix-cli -u dev get-book-levels -m 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8 -l 5`
```
          12.030  109.725
          12.015   66.583
          12.005   29.987
          11.995   15.006
          11.990    4.838
   5.843  11.980         
  15.031  11.975         
  30.087  11.965         
  66.917  11.955         
 110.552  11.940  
```
### get-full-book
Returns the full orderbook for a given market.

`$ phoenix-cli % phoenix-cli -u dev get-full-book -m 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8`
```
          12.210  409.500
          12.025  166.320
          12.005  109.954
          11.990   96.747
          11.980   15.025
          11.975    5.845
   5.850  11.965         
  15.050  11.960         
  30.125  11.950         
  67.057  11.930         
 110.784  11.915         
 168.137  11.895         
 426.985  11.710         
```

### get-transaction
Returns a summary of the market events that occured (Place, Fill, Reduce/Cancel) in a given transaction signature for the given market. 

`$ phoenix-cli -u dev get-transaction -m 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8 -s 4AA2WecMCJ3AsXJw2onLnihdpeCYw7tGDVHexMY37civAdyUnk8uP2aiiz64LQrLMBt4B62csYiFKQGWN33PuKyJ`
```
market: 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8, event_type: Place, timestamp: 1671652667, signature: 4AA2WecMCJ3AsXJw2onLnihdpeCYw7tGDVHexMY37civAdyUnk8uP2aiiz64LQrLMBt4B62csYiFKQGWN33PuKyJ, slot: 183671273, sequence_number: 1488049, event_index: 0, maker: mkrc4jMLEPRoKLUnNL7Ctnwb7uJykbwiYvFjB4sw9Z9, taker: , price: 11.895, side: Bid, quantity: 168.137
market: 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8, event_type: Place, timestamp: 1671652667, signature: 4AA2WecMCJ3AsXJw2onLnihdpeCYw7tGDVHexMY37civAdyUnk8uP2aiiz64LQrLMBt4B62csYiFKQGWN33PuKyJ, slot: 183671273, sequence_number: 1488050, event_index: 0, maker: mkrc4jMLEPRoKLUnNL7Ctnwb7uJykbwiYvFjB4sw9Z9, taker: , price: 12.21, side: Ask, quantity: 409.5
market: 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8, event_type: Place, timestamp: 1671652667, signature: 4AA2WecMCJ3AsXJw2onLnihdpeCYw7tGDVHexMY37civAdyUnk8uP2aiiz64LQrLMBt4B62csYiFKQGWN33PuKyJ, slot: 183671273, sequence_number: 1488051, event_index: 0, maker: mkrc4jMLEPRoKLUnNL7Ctnwb7uJykbwiYvFjB4sw9Z9, taker: , price: 11.71, side: Bid, quantity: 426.985
```

### get-market-status
Returns the status of a given market. Markets can be in the following states: Active, PostOnly, Paused, Closed, Uninitialized, Tombstoned.

`$ phoenix-cli -u dev get-market-status -m 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8`
```
Market status: Active
```

### get-seat-info
Returns the status and address of a trader's seat. By default, returns the payer's seat info. Seats can be in the following states: Approved, NotApproved, Retired

`$ phoenix-cli -u dev get-seat-info -m 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8 -t mkrc4jMLEPRoKLUnNL7Ctnwb7uJykbwiYvFjB4sw9Z9`
```
Seat address: mg6uXraBkvi7ccbnKvJoXgyjUVDBSqcxDJysPqiMYau
Seat status: Approved
```

### get-open-orders
Returns all open orders on a given market for a trader. By default, returns the payer's open orders. Returns the side, orderID, price in ticks, price, and size for each order. 

`$ phoenix-cli -u dev get-open-orders -m 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8 -t mkrc4jMLEPRoKLUnNL7Ctnwb7uJykbwiYvFjB4sw9Z9`
```
Open Bids
ID                   | Price (ticks)        | Price      | Quantity  
18446744073708399662 | 2385                 | 11.925     | 5.870     
18446744073708399660 | 2384                 | 11.920     | 15.100    
18446744073708399658 | 2382                 | 11.910     | 30.226    
18446744073708399656 | 2380                 | 11.900     | 67.226    
18446744073708399654 | 2377                 | 11.885     | 111.064   

Open Asks
ID                   | Price (ticks)        | Price      | Quantity  
1151952              | 2387                 | 11.935     | 4.865     
1151954              | 2388                 | 11.940     | 15.075    
1151956              | 2390                 | 11.950     | 30.125    
1151958              | 2392                 | 11.960     | 66.889    
1151960              | 2395                 | 11.975     | 110.229   
1151962              | 2399                 | 11.995     | 166.736   
```

### request-seat
Requests a seat for the payer on the given market. Note that the seat will have to then be approved by the market authority in order to place limit orders. 

`$ phoenix-cli -u dev request-seat -m 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8`
```
Requested seat, transaction signature: gbu5aqybhciNLamj5E9Bb1Xof387wTE1UJGJCT1cn7AHX7mQ1Air6bvSKNnoH7Hm3pq1JTQCDAcmyixEyQeS6FH
```

### mint-tokens
Mints tokens of the ticker_string (example: SOL) to the given pubkey. Default amount is 100_000_000_000. 

`$ phoenix-cli -u dev mint-tokens -m SOL -a 100000 -r aChXgDyJn7g5BCkjccisGc78LrQZKEmNgt5sz8Tdkzn`
```
Creating ATA
100000 Tokens minted! Mint pubkey: B1sL3zxwyVnDGzRWCAsBkjL23wyu8HgwQP4XxgnHiSrv,  Recipient address: aChXgDyJn7g5BCkjccisGc78LrQZKEmNgt5sz8Tdkzn
```

### mint-tokens-for-market
Mints the base and quote tokens of the given market to the given pubkey. Default amounts are 100_000_000_000 for base and 100_000_000 for quote.

`$ phoenix-cli -u dev mint-tokens-for-market -m 5iLqmcg8vifdnnw6wEpVtQxFE4Few5uiceDWzi3jvzH8 -r aChXgDyJn7g5BCkjccisGc78LrQZKEmNgt5sz8Tdkzn`
```
Creating ATA for quote token
Tokens minted! Signature: 3fnHTcRTfSrU4ycoE94p2Pn4zCA1389Xmr8uB8QvyYEPxWJWxfDiCa4Upp375Pqc7QC7pUg246dRMCp7PAbMRGnz
```












use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

// A static mapping of token tickers to pubkeys, with persistent storage.
// Meant for localhost development and should be cleared with each restart of the local validator
pub struct TokenRegistry {
    // Token pubkey is the key and the token ticker is the value
    pub pubkey_map: HashMap<Pubkey, String>,
    // Token pubkey is the value and the token ticker is the key. TODO: Better way to implement?
    pub ticker_map: HashMap<String, Pubkey>,
    pub path: PathBuf,
}

impl TokenRegistry {
    pub fn new(path: PathBuf) -> Self {
        TokenRegistry {
            pubkey_map: HashMap::new(),
            ticker_map: HashMap::new(),
            path,
        }
    }

    pub fn insert_record(&mut self, pubkey: Pubkey, ticker: String) -> anyhow::Result<()> {
        // Persist in-memory
        self.pubkey_map.insert(pubkey, ticker.clone());
        self.ticker_map.insert(ticker.clone(), pubkey);

        let pubkey_string = pubkey.to_string();
        // Persist in file system
        let token_record = TokenRecord::new(pubkey_string, ticker);

        let file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&self.path)?;

        serde_json::to_writer(file, &token_record)?;
        Ok(())
    }

    pub fn get_ticker_by_pubkey(&self, pubkey: &Pubkey) -> Option<String> {
        self.pubkey_map.get(pubkey).cloned()
    }

    pub fn get_pubkey_by_ticker(&self, ticker: &String) -> Option<Pubkey> {
        self.ticker_map.get(ticker).cloned()
    }

    pub fn open(path: impl Into<PathBuf>) -> anyhow::Result<TokenRegistry> {
        let path_buf: PathBuf = path.into();
        let file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path_buf.as_path())?;

        //Read existing tokens into memory
        let deserialized_records: Vec<TokenRecord> = serde_json::Deserializer::from_reader(file)
            .into_iter::<TokenRecord>()
            .filter_map(|it| it.ok())
            .collect::<_>();

        let mut in_mem_registry = TokenRegistry::new(path_buf);

        for record in deserialized_records.iter() {
            if let Ok(pubkey) = Pubkey::from_str(&record.pubkey) {
                in_mem_registry.insert_record(pubkey, record.ticker.clone())?;
            } else {
                continue;
            }
        }

        Ok(in_mem_registry)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenRecord {
    pubkey: String,
    ticker: String,
}

impl TokenRecord {
    fn new(pubkey: String, ticker: String) -> Self {
        TokenRecord { pubkey, ticker }
    }
}

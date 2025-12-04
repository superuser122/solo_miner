use serde::{Serialize, Deserialize};
use std::{fs, io};

const CONFIG_FILE: &str = "miner_config.json";

/// Represents the static data needed to start mining a new block.
/// This data would normally come from a Bitcoin RPC call (getblocktemplate).
#[derive(Debug, Serialize, Deserialize)]
pub struct MinerSettings {
    /// Bitcoin protocol version (e.g., 536870912 or 0x20000000)
    pub version: u32,

    /// Hash of the previous block header (32 bytes, hex string)
    pub prev_block_hash: String,

    /// The compressed difficulty target (nBits, e.g., 0x1800ffff)
    /// This is the number that determines how many leading zeros the hash must have.
    pub nbits: u32,
    
    // The Bitcoin address to send the block reward to.
    pub reward_address: String,
    
    //The block reward in satoshis (e.g., 625,000,000 for 6.25 BTC)
    pub block_reward_sats: u64,
    
    /// Starting Unix timestamp (will be incremented during mining).
    pub timestamp: u32,
}

impl MinerSettings {
    /// Provides a reasonable default set of values for the current Bitcoin mainnet.
    /// NOTE: To actually attempt to solo mine, these values should be updated
    /// frequently from the current network state!
    pub fn default() -> Self {
        MinerSettings {
            version: 0x20000000, // Common version for signaling
            // A common block hash from late 2025/early 2026 for demonstration
            prev_block_hash: "000000000000000000021a8c91d4e78f9f23b123456789abcdef0123456789".to_string(),
            // A typical nBits value for Bitcoin (around difficulty 18.0)
            nbits: 0x1800ffff, 
            // NOTE: REPLACE THIS WITH YOUR OWN ADDRESS (e.g., a testnet address)
            reward_address: "1M8QW8P9Bf98R71822sF8T2L8J4H7Q7wM2".to_string(), 
            block_reward_sats: 625000000, // 6.25 BTC
            // Current Unix time (to be updated on load)
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as u32,
        }
    }

    /// Saves the current settings structure to a JSON file.
    pub fn save(&self) -> io::Result<()> {
        let json_data = serde_json::to_string_pretty(self)?;
        fs::write(CONFIG_FILE, json_data)?;
        println!("Configuration saved to {CONFIG_FILE}");
        Ok(())
    }

    /// Attempts to load settings from the JSON file. If the file is missing,
    /// it creates a default configuration, saves it, and then returns it.
    pub fn load() -> io::Result<Self> {
        match fs::read_to_string(CONFIG_FILE) {
            Ok(data) => {
                let settings: MinerSettings = serde_json::from_str(&data)?;
                println!("Configuration loaded from {CONFIG_FILE}");
                Ok(settings)
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                println!("Configuration file not found. Creating default...");
                let default_settings = MinerSettings::default();
                default_settings.save()?;
                Ok(default_settings)
            },
            Err(e) => Err(e),
        }
    }
}
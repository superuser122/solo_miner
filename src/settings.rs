use serde::{Serialize, Deserialize};
use std::{fs, io};
use std::time::SystemTime;

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

    // --- Fields for connecting to Bitcoin Core RPC ---
    /// The RPC URL of your Bitcoin node (e.g., "http://127.0.0.1:8332")
    pub rpc_url: String,
    /// The RPC username you configured for your Bitcoin node.
    pub rpc_user: String,
    /// The RPC password you configured for your Bitcoin node.
    #[serde(skip_serializing)] // Don't save the password to the config file
    pub rpc_pass: String,
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
            reward_address: "bc1q...".to_string(), 
            block_reward_sats: 625000000, // 6.25 BTC
            // Current Unix time (to be updated on load)
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as u32,
            rpc_url: "http://127.0.0.1:8332".to_string(),
            rpc_user: "your_rpc_user".to_string(),
            rpc_pass: "your_rpc_password".to_string(),
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
                let mut settings: MinerSettings = serde_json::from_str(&data)?;
                // The password is not saved, so we prompt for it on load.
                println!("Please enter your Bitcoin Core RPC password:");
                settings.rpc_pass = rpassword::prompt_password("Password: ")?;
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

    /// Fetches the latest block template from a Bitcoin node and updates settings.
    pub fn update_from_node(&mut self) -> io::Result<()> {
        println!("\n[RPC] Contacting Bitcoin node to get new block template...");

        // 1. Define structs for parsing the RPC response.
        #[derive(Deserialize)]
        struct RpcResponse<T> {
            result: T,
            // We ignore the 'error' and 'id' fields for this simple case.
        }

        #[derive(Deserialize)]
        struct GetBlockTemplateResult {
            previousblockhash: String,
            coinbasevalue: u64,
            bits: String,
        }

        // 2. Create a blocking HTTP client.
        let client = reqwest::blocking::Client::new();

        // 3. Construct the JSON-RPC request body.
        let request_body = serde_json::json!({
            "jsonrpc": "1.0",
            "id": "solo-miner",
            "method": "getblocktemplate",
            "params": [{"rules": ["segwit"]}]
        });

        // 4. Send the request with basic authentication.
        let response = client.post(&self.rpc_url)
            .basic_auth(&self.rpc_user, Some(&self.rpc_pass))
            .json(&request_body)
            .send()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("RPC request failed: {}", e)))?;

        // 5. Parse the response and update the settings.
        if response.status().is_success() {
            let rpc_response: RpcResponse<GetBlockTemplateResult> = response.json()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse RPC JSON response: {}", e)))?;
            
            let template = rpc_response.result;

            self.prev_block_hash = template.previousblockhash;
            self.block_reward_sats = template.coinbasevalue;
            self.nbits = u32::from_str_radix(&template.bits, 16)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse nbits hex: {}", e)))?;
            self.timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as u32;
            
            println!("[RPC] Successfully updated block template.");
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, format!("RPC Error: {} - {}", response.status(), response.text().unwrap_or_default())))
        }
    }
}
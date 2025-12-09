use crate::settings::MinerSettings;
use std::{io, str::FromStr, time};
use std::io::Write;
use bitcoin::Network;
use bitcoin::consensus::Encodable;
use bitcoin::hashes::Hash;
use sha2::{Digest, Sha256};
use byteorder::{LittleEndian, WriteBytesExt};
use hex;


const COINBASE_DATA: &[u8] = b"/solo-miner/rust-pi-edu/";

/// Helper function to perform Double SHA-256 (SHA-256d)
pub fn sha256d(data: &[u8]) -> [u8; 32] {
    let hash1 = Sha256::digest(data);
    let hash2 = Sha256::digest(&hash1);
    
    // The hash is returned in big-endian order 
    hash2.into()
}

/// Converts the compact difficulty (nBits) into the full 256-bit target hash.
/// The mining hash must be LESS THAN this target.
fn compact_to_target_old(nbits: u32) -> [u8; 32] {
    let mut target = [0u8; 32];
    let nbits_bytes = nbits.to_be_bytes();
    
    // The first byte is the exponent (size of the target in bytes)
    let size = nbits_bytes[0]; 
    // The next three bytes are the mantissa (significand)
    let mantissa = u32::from_be_bytes([0, nbits_bytes[1], nbits_bytes[2], nbits_bytes[3]]);

    if size > 3 {
        // We write the mantissa bytes backwards (due to big-endian representation of the number)
        for i in 0..3 {
            let byte_index = (size as usize) - 1 - i;
            if byte_index < 32 {
                target[31 - byte_index] = (mantissa >> (i * 8)) as u8;
            }
        }
    }
    target
}

/// Converts compact nBits to a 32-byte big-endian target.
fn compact_to_target(nbits: u32) -> [u8; 32] {
    let mut target = [0u8; 32];
    let exponent = (nbits >> 24) as i32;
    // Mask off the sign bit for standard mantissa representation
    let mut mantissa = nbits & 0x007fffff; 

    if exponent <= 3 {
        // Handle targets smaller than 4 bytes (shift the mantissa right)
        let shift = 8 * (3 - exponent);
        mantissa >>= shift as u32;
        // Place the 4 bytes of the (padded) mantissa at the end
        target[28..32].copy_from_slice(&mantissa.to_be_bytes());
    } else {
        // Handle targets larger than 4 bytes (place mantissa at the appropriate offset)
        let byte_pos = (32 - (exponent as usize)) as isize;
        // mantissa as 3 bytes big-endian
        let mant_bytes = [(mantissa >> 16) as u8, (mantissa >> 8) as u8, mantissa as u8];
        if byte_pos >= 0 && (byte_pos as usize) + 3 <= 32 {
            target[(byte_pos as usize)..(byte_pos as usize + 3)].copy_from_slice(&mant_bytes);
        }
    }

    // This target is now in BIG-ENDIAN order
    target 
}

/// Creates a minimal Coinbase Transaction and returns its double SHA-256 hash,
/// which serves as the Merkle Root for a block containing only this transaction.
fn calculate_merkle_root(reward_address: &str, block_reward_sats: u64) -> io::Result<([u8; 32], bitcoin::Transaction)> {
    // 1. Decode the reward address to get the scriptPubKey
    // First, parse the string into an unchecked address.
    let address = bitcoin::Address::from_str(reward_address)
        // Then, require that the address is valid for the Bitcoin main network.
        .and_then(|addr| addr.require_network(Network::Bitcoin))
        .map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid or non-mainnet Bitcoin address: {}", e))
        })?;
    let script_pubkey = address.script_pubkey(); // Now we use the network-checked address.

    // 2. Build the Coinbase Transaction (minimal, non-standard)
    let tx = bitcoin::Transaction {
        version: bitcoin::transaction::Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![
            bitcoin::TxIn {
                previous_output: bitcoin::OutPoint::null(),
                script_sig: bitcoin::ScriptBuf::from_bytes(COINBASE_DATA.to_vec()),
                sequence: bitcoin::transaction::Sequence::MAX,
                witness: bitcoin::Witness::new(), 
            },
        ],
        output: vec![
            bitcoin::TxOut {
                value: bitcoin::Amount::from_sat(block_reward_sats),
                script_pubkey: script_pubkey,
            },
        ],
    };

    // 3. Serialize and Double Hash the transaction to get the Merkle Root
    let mut serialized_tx = Vec::new();
    tx.consensus_encode(&mut serialized_tx).map_err(|e| {
         io::Error::new(io::ErrorKind::Other, format!("Failed to serialize tx: {}", e))
    })?;

    let merkle_root_hash = sha256d(&serialized_tx);

    Ok((merkle_root_hash, tx))
}

/// Assembles the 80-byte block header and starts the high-speed hashing loop.
pub fn mine_block(settings: MinerSettings) -> io::Result<()> {
    println!("\n[Mining] Initializing Block...");
    
    // Convert hex strings to byte arrays
    let mut prev_hash_bytes = hex::decode(&settings.prev_block_hash).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    prev_hash_bytes.reverse();
    let (mut merkle_root_bytes, coinbase_tx) = calculate_merkle_root(&settings.reward_address, settings.block_reward_sats)?;
    merkle_root_bytes.reverse();
    let mut target = compact_to_target(settings.nbits);

    target.reverse();

    let mut nonce: u32 = 0;
    let mut hash_rate_start = time::Instant::now();
    let mut hash_count: u64 = 0;

    println!("[Mining] Target Hash (Little Endian): {}", hex::encode(&target));

    loop {
        // 1. Construct the 80-byte Block Header
        let mut block_header = [0u8; 80];
        let mut cursor = io::Cursor::new(&mut block_header[..]);

        // All values are written in Little-Endian byte order
        cursor.write_u32::<LittleEndian>(settings.version)?;
        cursor.write_all(&prev_hash_bytes)?; 
        cursor.write_all(&merkle_root_bytes)?;
        
        let current_time = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs() as u32;
        cursor.write_u32::<LittleEndian>(current_time)?;
        
        cursor.write_u32::<LittleEndian>(settings.nbits)?;
        cursor.write_u32::<LittleEndian>(nonce)?; // The variable we are changing

        // 2. Perform Double SHA-256
        let mut block_hash = sha256d(&block_header);
        hash_count += 1;
        //it will be the Little-Endian protocol hash
        block_hash.reverse();
        // 3. Check Difficulty: Compare the hash against the target
        if block_hash.lt(&target) {
            println!("\n==============================================");
            println!("🎉 BLOCK FOUND! (The Lottery is Won!)");
            println!("Hash: {}", hex::encode(&block_hash));
            println!("Nonce: {}", nonce);
            println!("==============================================");

            match serialize_block(
                &settings, 
                &prev_hash_bytes, 
                &merkle_root_bytes, 
                nonce, 
                current_time,
                &coinbase_tx 
            ) {
                Ok(block_hex) => {
                    // Call the RPC submission function
                    submit_block_to_node(&settings,&block_hex)?; 
                },
                Err(e) => {
                    eprintln!("Error serializing block for submission: {}", e);
                    // We still break, but note the error.
                }
            }

            break;
        }

        // 4. Increment Nonce
        nonce = nonce.wrapping_add(1);

        // Periodically report Hash Rate
        if nonce % 1_000_000 == 0 {
            let elapsed = hash_rate_start.elapsed().as_secs_f64();
            let hashrate = hash_count as f64 / elapsed / 1_000_000.0;
            println!("Status: Hashed {}M nonces. Hashrate: {:.3} MH/s", hash_count / 1_000_000, hashrate);
            hash_rate_start = time::Instant::now();
            hash_count = 0;
        }
        
        // If nonce overflows, the miner needs to get a new block template
        if nonce == 0 {
            println!("[Mining] Nonce overflowed! Stopping search in this template space.");
            return Ok(());
        }
    }

    Ok(())
}

// In src/miner.rs, add this new function before mine_block or after sha256d

/// Submits the raw, serialized block to the Bitcoin node via the submitblock RPC.
pub fn submit_block_to_node(
    settings: &MinerSettings, // Need to reference the settings struct
    block_hex: &str
) -> io::Result<()> {
    println!("\n[RPC] Submitting found block to node...");

    // 1. Define the RPC response structure for submission
    // Note: You may need to ensure serde::Deserialize is available in miner.rs
    // or pass the response handling to settings.rs if you want to keep
    // serde::Deserialize there. Since submit_block is simple, let's keep it here.
    use serde::Deserialize; 

    #[derive(Deserialize)]
    struct RpcResponse {
        result: Option<String>,
    }

    let client = reqwest::blocking::Client::new();

    // 2. Construct the JSON-RPC request body
    let request_body = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "solo-miner-submit",
        "method": "submitblock",
        "params": [block_hex] 
    });

    // 3. Send the authenticated request using fields from the passed settings reference
    let response = client.post(&settings.rpc_url)
        .basic_auth(&settings.rpc_user, Some(&settings.rpc_pass))
        .json(&request_body)
        .send()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("RPC submission request failed: {}", e)))?;

    // 4. Parse and check the response (rest of the logic remains the same)
    if response.status().is_success() {
        let rpc_response: RpcResponse = response.json()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse submitblock JSON response: {}", e)))?;
        
        if let Some(err_message) = rpc_response.result {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Block submission rejected by node: {}", err_message)));
        }
        
        println!("✅ [RPC] Block submitted successfully! Check your node logs.");
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, format!("RPC Submission Error: {} - {}", response.status(), response.text().unwrap_or_default())))
    }
}

/// Creates the full block structure, serializes it, and returns the hex string.
fn serialize_block(
    settings: &crate::settings::MinerSettings, 
    prev_hash_bytes: &[u8], 
    merkle_root_bytes: &[u8], 
    nonce: u32, 
    current_time: u32,
    // NEW PARAMETER: Accept the pre-built Coinbase Transaction
    coinbase_tx: &bitcoin::Transaction 
) -> io::Result<String> {
    
    let prev_blockhash = bitcoin::block::BlockHash::from_slice(prev_hash_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid prev hash: {}", e)))?;
    let merkle_root = bitcoin::TxMerkleNode::from_slice(merkle_root_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid merkle root: {}", e)))?;

    let block_header = bitcoin::block::Header {
        version: bitcoin::block::Version::from_consensus(settings.version as i32),
        prev_blockhash,
        merkle_root,
        time: current_time,
        bits: bitcoin::CompactTarget::from_consensus(settings.nbits),
        nonce,
    };


    // Create the full Block structure
    let block = bitcoin::Block {
        header: block_header,
        // Use the passed transaction reference
        txdata: vec![coinbase_tx.clone()], 
    };

    // Serialize the block into a Vec<u8>
    let mut serialized_block = Vec::new();
    block.consensus_encode(&mut serialized_block).map_err(|e| {
         io::Error::new(io::ErrorKind::Other, format!("Failed to serialize block: {}", e))
    })?;

    // Return the result as a hex string for RPC submission
    Ok(hex::encode(&serialized_block))
}
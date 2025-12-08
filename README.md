# ⛏️ Solo Lottery Miner (Educational Project)

This project is a simple, command-line Bitcoin mining application built in **Rust**. It is designed strictly for **educational purposes**—specifically, to demonstrate and explore the core components of the Bitcoin Proof-of-Work (PoW) consensus mechanism, often referred to as "Lottery Mining."

**⚠️ Disclaimer:** This software will not realistically find a block on the main Bitcoin network due to the current, extreme network difficulty and the use of CPU-based hashing (compared to specialized ASIC miners). It is intended solely for learning cryptographic and block construction concepts.

## ✨ Project Goals and Features

* **Proof-of-Work Implementation:** Implements the core **Double SHA-256 (SHA-256d)** hashing loop.
* **Block Header Construction:** Demonstrates how the 80-byte Bitcoin block header (Version, Previous Hash, Merkle Root, Timestamp, nBits, Nonce) is constructed and serialized.
* **Difficulty Target:** Parses and compares the resulting hash against the Bitcoin difficulty target (`nBits`).
* **Coinbase Transaction:** Integrates a basic Coinbase transaction using a hardcoded reward address, essential for calculating the Merkle Root.
* **Resource Efficiency:** Built in Rust for low-level performance suitable for low-power CPUs (like those found in a Raspberry Pi).
* **Configuration:** Uses JSON serialization (`serde`) for external management of mining parameters.

## ⚙️ Core Technology

* **Language:** Rust
* **Key Crates:** `sha2`, `byteorder`, `serde`, `bitcoin` (for utilities)
* **Target Device:** CPU (tested on low-power devices)

## 🚀 Getting Started (for Educational Setup)

### Prerequisites

1.  **Rust Toolchain:** Installed via `rustup`.
2.  **Bitcoin Core Node:** A fully synchronized Bitcoin Core instance running on the same network.
3.  **RPC Credentials:** Access credentials configured in your node's `bitcoin.conf`.

### Setup Steps

1.  **Clone the Repository:**
    ```bash
    git clone https://github.com/superuser122/solo_miner.git
    cd solo_miner 
    ```

2.  **Configure Bitcoin Core (Mandatory):**
    Edit your node's `bitcoin.conf` file to enable RPC access and set a username/password.
    ```ini
    server=1
    rpcallowip=127.0.0.1  # Or the IP of the miner machine
    rpcuser=your_custom_user
    rpcpassword=a_secure_password
    ```

3.  **Build the Project:**
    ```bash
    cargo build --release
    ```
    *(Use `--release` for optimal hashing performance.)*

4.  **Run the Miner and Configure RPC:**
    The first time you run it, the file `miner_config.json` will be created.

    ```bash
    cargo run --release
    ```

    * **Edit `miner_config.json`:** Update the **`rpc_url`**, **`rpc_user`**, and crucially, set your **`reward_address`**.
    * **Enter Password:** The application will prompt you securely for the **`rpc_pass`** at runtime (it is never saved to disk).

5.  **Start Mining and Current Limitations:**

    The application will connect to the node, fetch the latest block data, and begin the high-speed hashing loop.

    ⚠️ **Important Limitation (Next Feature):**

    If your miner finds a valid **Nonce** that solves the Proof-of-Work puzzle:

    * It will correctly announce the block as found and print the Nonce and Hash.
    * **It does NOT currently submit the block** to the Bitcoin network. The application will stop.
    * The necessary RPC implementation (the `submitblock` call) will be implemented in the next development step.
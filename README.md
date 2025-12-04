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

* Rust toolchain (installed via `rustup`)

### Running the Miner

1.  Clone the repository:
    ```bash
    git clone [YOUR-REPO-LINK]
    cd solo_lottery_miner
    ```
2.  Build and run the application:
    ```bash
    cargo run
    ```
    *(The first run will generate a `miner_config.json` file where you can specify your version, target difficulty, and reward address.)*
3.  Modify the generated `miner_config.json` with the parameters you want to test.

---
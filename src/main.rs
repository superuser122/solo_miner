mod settings;
mod miner;


fn main() {
    match settings::MinerSettings::load() {
        Ok(mut settings) => {
            println!("\n--- Loaded Miner Settings ---");
            println!("{:#?}", settings);
            println!("-----------------------------");

            // Update settings with live data from the Bitcoin node
            if let Err(e) = settings.update_from_node() {
                eprintln!("Could not update settings from node: {}. Check RPC settings in miner_config.json.", e);
                return;
            }
            if let Err(e) = miner::mine_block(settings) {
                eprintln!("A critical mining error occurred: {}", e);
            }
        },
        Err(e) => eprintln!("Failed to load or save configuration: {}", e),
    }
}
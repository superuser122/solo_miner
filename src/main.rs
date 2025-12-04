mod settings;

fn main() {
    match settings::MinerSettings::load() {
        Ok(settings) => {
            println!("\n--- Loaded Miner Settings ---");
            println!("{:#?}", settings);
            println!("-----------------------------");
            // The actual mining logic will start here in the next step
        },
        Err(e) => eprintln!("Failed to load or save configuration: {}", e),
    }
}
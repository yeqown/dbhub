use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "configs/"]
pub struct Configs;

#[derive(RustEmbed)]
#[folder = "scripts/"]
pub struct Scripts;

#[allow(unused)]
pub fn debug_embed() {
    println!("Configs:");
    for file in Configs::iter() {
        println!("  {}", file);
    }

    println!("Scripts:");
    for file in Scripts::iter() {
        println!("  {}", file);
    }
}
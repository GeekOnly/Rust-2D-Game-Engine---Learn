use anyhow::Result;

fn main() -> Result<()> {
    println!("🎮 Rust 2D Game Engine");
    println!();
    println!("Available commands:");
    println!("  cargo run --bin player    - Run the game player");
    println!("  cargo run -p editor       - Run the game editor");
    println!();
    println!("For game development, use the editor:");
    println!("  cargo run -p editor");
    println!();
    println!("To play exported games, use the player:");
    println!("  cargo run --bin player");
    
    Ok(())
}
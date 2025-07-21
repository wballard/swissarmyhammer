use swissarmyhammer::issues::filesystem::parse_any_issue_filename;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Debugging parse_any_issue_filename for 000186 ===");
    
    let filename = "000186";
    println!("Input filename: '{filename}'");
    
    match parse_any_issue_filename(filename) {
        Ok((number, name)) => {
            println!("✅ Parse successful:");
            println!("  Number: {number:?}");
            println!("  Name: '{name}'");
            println!("  Name is empty: {}", name.is_empty());
        }
        Err(e) => {
            println!("❌ Parse failed: {e}");
        }
    }
    
    // Also test the numbered vs non-numbered format logic
    println!("\n=== Understanding the parsing logic ===");
    println!("Testing parse_issue_filename first...");
    
    Ok(())
}
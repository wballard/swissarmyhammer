fn main() {
    // Configure logging to stderr
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    println!("Hello, swissarmyhammer!");
}

//! Print the resolved (secret-redacted) provider configuration, so you can verify your
//! environment / `.env` is picked up before running live evaluations.
//!
//! Run: `cargo run --example show_config`

use ragas::ProviderConfig;

fn main() {
    let config = ProviderConfig::from_env();

    println!("Resolved ragas provider configuration (env > .env > default):\n");
    println!("{}", config.redacted_summary());
    println!();

    if config.has_api_key() {
        println!("✓ chat API key is set — LLM-based metrics, testset generation and the CLI run live.");
    } else {
        println!(
            "✗ no chat API key — LLM features stay offline.\n  Set OPENAI_API_KEY in your shell or in a .env file (see .env.example)."
        );
    }
}

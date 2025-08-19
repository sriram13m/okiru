use chrono::Utc;
use okiru::{ActivityLogger, AppInfo, MonitorConfig, OllamaClient, start_monitoring};
use tokio;

async fn test_ollama() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Ollama integration...");

    let client = OllamaClient::new(
        "http://localhost:11434".to_string(),
        "gemma3:4b".to_string(),
    );

    let test_app = AppInfo {
        app_name: "Visual Studio Code".to_string(),
        window_title: "main.rs - okiru".to_string(),
        bundle_id: "com.microsoft.VSCode".to_string(),
        process_id: 12345,
    };

    let result = client
        .evaluate_alignment(
            "I want to learn Rust programming for 2 hours today",
            &test_app,
            "",
        )
        .await?;

    println!("Alignment Score: {:.2}", result.alignment_score);
    println!("Reasoning: {}", result.reasoning);
    if let Some(suggestion) = result.suggestion {
        println!("Suggestion: {}", suggestion);
    }
    println!("Confidence: {:.2}", result.confidence);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    test_ollama().await?;

    println!("\nStarting monitoring...");
    let logger = ActivityLogger::new("sqlite:activity.db").await?;

    start_monitoring(MonitorConfig::default(), logger, |application_info| {
        let timestamp = Utc::now();
        println!("{}: {}", timestamp.format("%H:%M:%S"), application_info);
    })
    .await?;

    Ok(())
}

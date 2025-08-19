use chrono::Utc;
use okiru::{ActivityLogger, MonitorConfig, start_monitoring};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Okiru - Activity Monitoring with AI Evaluation");

    let logger = ActivityLogger::new("sqlite:activity.db").await?;

    start_monitoring(MonitorConfig::default(), logger, |application_info| {
        let timestamp = Utc::now();
        println!("{}: {}", timestamp.format("%H:%M:%S"), application_info);
    })
    .await?;

    Ok(())
}

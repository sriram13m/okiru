use tokio;
use chrono::Utc;
use okiru::{start_monitoring, MonitorConfig, ActivityLogger, AppInfo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting monitoring");

    let logger = ActivityLogger::new("sqlite:activity.db").await?;

    start_monitoring(MonitorConfig::default(), logger, |application_info| {
        let timestamp = Utc::now();
        println!("{}: {}", timestamp.format("%H:%M:%S"), application_info);
    }).await?;

    Ok(())
}

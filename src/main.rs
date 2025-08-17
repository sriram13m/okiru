use chrono::Utc;
use okiru::{start_monitoring, MonitorConfig};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting monitoring");

    start_monitoring(MonitorConfig::default(), |app_info| {
        let timestamp = Utc::now();
        println!("{} {}", timestamp.format("%H:%M:%S"), app_info);
    })?;

    Ok(())
}

use objc2::runtime::AnyObject;
use objc2::{ClassType, msg_send};
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_core_foundation::{CFRunLoop, kCFRunLoopDefaultMode};
use objc2_foundation::NSString;
use std::fmt::write;
use std::time::Duration;
use tokio::task::try_id;

use crate::ActivityLogger;

//App Info
#[derive(Debug)]
pub struct AppInfo {
    pub app_name: String,
    pub window_title: String,
    pub bundle_id: String,
    pub process_id: i32,
}

impl std::fmt::Display for AppInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "App: {} | Window: {} | Bundle: {} | PID: {}",
            self.app_name, self.window_title, self.bundle_id, self.process_id
        )
    }
}

//MonitorError

#[derive(Debug)]
pub enum MonitorError {
    NoActiveApp,
    ApiError(String),
}

impl std::fmt::Display for MonitorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorError::NoActiveApp => write!(f, "No Active Application found"),
            MonitorError::ApiError(msg) => write!(f, "API error: {}", msg),
        }
    }
}

impl std::error::Error for MonitorError {}

//Utils
//
unsafe fn nsstring_to_string(ns_string_ptr: *const NSString, fallback: &str) -> String {
    if ns_string_ptr.is_null() {
        fallback.to_string()
    } else {
        (*ns_string_ptr).to_string()
    }
}

fn frontmost_application_to_app_info(
    frontmost_application: Option<objc2::rc::Retained<objc2_app_kit::NSRunningApplication>>,
) -> Result<AppInfo, MonitorError> {
    unsafe {
        let application = frontmost_application.ok_or(MonitorError::NoActiveApp)?;

        let application_name: *const NSString = msg_send![&*application, localizedName];
        let bundle_id: *const NSString = msg_send![&*application, bundleIdentifier];
        let process_id: i32 = msg_send![&*application, processIdentifier];
        let window_title = format!(
            "{} - Window",
            nsstring_to_string(application_name, "Unknown Application")
        );

        Ok(AppInfo {
            app_name: nsstring_to_string(application_name, "Unknown Application"),
            bundle_id: nsstring_to_string(bundle_id, "Unknown Bundle Id"),
            process_id: process_id,
            window_title: window_title,
        })
    }
}

//Get Active Window Function

pub fn get_active_window() -> Result<AppInfo, MonitorError> {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let frontmost_application = msg_send![&*workspace, frontmostApplication];
        frontmost_application_to_app_info(frontmost_application)
    }
}

//Monitoring Function
//Utils

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub poll_interval_ms: u64,
    pub runloop_timeout: f64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 1000,
            runloop_timeout: 0.1,
        }
    }
}

//Function
pub async fn start_monitoring<F>(
    config: MonitorConfig,
    mut logger: ActivityLogger,
    callback: F,
) -> Result<(), MonitorError>
where
    F: Fn(&AppInfo),
{
    let mut last_application: Option<AppInfo> = None;

    loop {
        match get_active_window() {
            Ok(current_application) => {
                callback(&current_application);

                let app_changed = match &last_application {
                    None => true,
                    Some(last) => last.bundle_id != current_application.bundle_id,
                };

                if app_changed {
                    if let Err(e) = logger.start_session(&current_application).await {
                        eprintln!("Storage error: {}", e);
                    }

                    last_application = Some(current_application);
                }
            }
            Err(e) => eprintln!("Error while monitoring: {}", e),
        }

        tokio::time::sleep(Duration::from_millis(config.poll_interval_ms)).await;

        unsafe {
            CFRunLoop::run_in_mode(kCFRunLoopDefaultMode, config.runloop_timeout, true);
        }
    }
}

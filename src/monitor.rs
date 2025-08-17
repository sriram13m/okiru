use std::fmt::write;
use objc2::runtime::AnyObject;
use objc2::{msg_send, ClassType};
use objc2_foundation::NSString;
use objc2_app_kit::{NSWorkspace, NSRunningApplication};

#[derive(Debug)]
pub struct AppInfo {
    pub app_name: String,
    pub window_title: String,
    pub bundle_id: String,
    pub process_id: i32,
}

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

unsafe fn nsstring_to_string(ns_string_ptr: *const NSString, fallback: &str) -> String {
    if ns_string_ptr.is_null() {
        fallback.to_string()
    } else {
        (*ns_string_ptr).to_string()
    }
}

fn frontmost_application_to_app_info(frontmost_application: Option<objc2::rc::Retained<objc2_app_kit::NSRunningApplication>>) -> Result<AppInfo, MonitorError> {
    unsafe {
        let application = frontmost_application.ok_or(MonitorError::NoActiveApp)?;

        let application_name: *const NSString = msg_send![&*application, localizedName];
        let bundle_id: *const NSString = msg_send![&*application, bundleIdentifier];
        let process_id: i32 = msg_send![&*application, processIdentifier];
        let window_title = format!("{} - Window", nsstring_to_string(application_name, "Unknown Application"));

        Ok(AppInfo {
            app_name: nsstring_to_string(application_name, "Unknown Application"),
            bundle_id: nsstring_to_string(bundle_id, "Unknown Bundle Id"),
            process_id: process_id,
            window_title: window_title,
            
        })

    }
}

pub fn get_active_window() -> Result<AppInfo, MonitorError> {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let frontmost_application = msg_send![&*workspace, frontmostApplication];
        frontmost_application_to_app_info(frontmost_application)
    }
}

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Serialize)]
pub struct StructuredLogEvent<'a, T: Serialize> {
    pub level: LogLevel,
    pub event: &'a str,
    pub request_id: &'a str,
    pub timestamp_ms: i64,
    pub data: &'a T,
}

/// Cloudflare Analytics Engine data point structure
#[derive(Debug, Serialize)]
pub struct AnalyticsDataPoint<'a> {
    pub dataset: &'a str,
    pub blobs: Vec<&'a str>,
    pub doubles: Vec<f64>,
}

pub fn log_event<T: Serialize>(event: &str, request_id: &str, data: &T) {
    log_structured(LogLevel::Info, event, request_id, data);
}

pub fn log_structured<T: Serialize>(level: LogLevel, event: &str, request_id: &str, data: &T) {
    let envelope = StructuredLogEvent {
        level,
        event,
        request_id,
        timestamp_ms: worker::Date::now().as_millis() as i64,
        data,
    };
    if let Ok(json) = serde_json::to_string(&envelope) {
        match level {
            LogLevel::Error => worker::console_error!("{}", json),
            LogLevel::Warn => worker::console_warn!("{}", json),
            LogLevel::Info => worker::console_log!("{}", json),
        }
    }
}

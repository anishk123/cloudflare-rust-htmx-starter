use serde::Serialize;

pub fn log_event<T: Serialize>(event: &str, request_id: &str, data: &T) {
    let envelope = serde_json::json!({"event": event, "requestId": request_id, "data": data});
    worker::console_log!("{}", envelope);
}

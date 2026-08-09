use uuid::Uuid;
pub fn request_id() -> String {
    Uuid::new_v4().to_string()
}

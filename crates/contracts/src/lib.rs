use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CONTRACT_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub status: NoteStatus,
    pub version: u32,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoteStatus { Draft, Published, Archived }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CreateNoteRequest {
    pub title: String,
    pub body: String,
    pub operation_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CreateNoteResponse { pub note: Note }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SyncOperationV1 {
    pub contract_version: u8,
    pub operation_id: Uuid,
    pub entity_id: Option<Uuid>,
    pub base_version: Option<u32>,
    pub action: SyncAction,
    pub payload: serde_json::Value,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncAction { CreateNote, UpdateNote }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SyncBatchRequest { pub operations: Vec<SyncOperationV1> }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SyncResult {
    pub operation_id: Uuid,
    pub accepted: bool,
    pub entity_id: Option<Uuid>,
    pub server_version: Option<u32>,
    pub conflict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SyncBatchResponse { pub results: Vec<SyncResult> }

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobV1 {
    SummarizeNote {
        contract_version: u8,
        job_id: Uuid,
        note_id: Uuid,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SummaryOutputV1 {
    pub contract_version: u8,
    pub summary: String,
    pub confidence: f32,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AttachmentMetadataV1 {
    pub contract_version: u8,
    pub key: String,
    pub content_type: String,
    pub size_bytes: u64,
}

pub fn contract_catalog_json() -> serde_json::Value {
    serde_json::json!({
        "contractVersion": CONTRACT_VERSION,
        "schemas": {
            "Note": schema_for!(Note),
            "CreateNoteRequest": schema_for!(CreateNoteRequest),
            "SyncOperationV1": schema_for!(SyncOperationV1),
            "JobV1": schema_for!(JobV1),
            "SummaryOutputV1": schema_for!(SummaryOutputV1),
            "AttachmentMetadataV1": schema_for!(AttachmentMetadataV1)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_contract_round_trips() {
        let job = JobV1::SummarizeNote { contract_version: 1, job_id: Uuid::nil(), note_id: Uuid::nil() };
        let value = serde_json::to_string(&job).unwrap();
        let decoded: JobV1 = serde_json::from_str(&value).unwrap();
        assert_eq!(job, decoded);
    }

    #[test]
    fn catalog_contains_durable_contracts() {
        let value = contract_catalog_json();
        assert!(value["schemas"]["JobV1"].is_object());
        assert!(value["schemas"]["SyncOperationV1"].is_object());
    }
}

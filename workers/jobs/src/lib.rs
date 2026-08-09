use starter_contracts::{JobV1, SummaryOutputV1, CONTRACT_VERSION};
use starter_domain::summarize_deterministically;
use starter_observability::log_event;
use worker::*;

fn now_ms() -> i64 {
    Date::now().as_millis() as i64
}

#[event(queue)]
pub async fn main(batch: MessageBatch<JobV1>, env: Env, _ctx: Context) -> Result<()> {
    let db = env.d1("DB")?;
    for message in batch.messages()? {
        match message.body() {
            JobV1::SummarizeNote {
                contract_version,
                job_id,
                note_id,
            } => {
                if *contract_version != CONTRACT_VERSION {
                    message.ack();
                    continue;
                }
                if starter_database::is_job_processed(&db, *job_id).await? {
                    message.ack();
                    continue;
                }
                let Some(note) = starter_database::find_note(&db, *note_id).await? else {
                    message.ack();
                    continue;
                };
                let output = SummaryOutputV1 {
                    contract_version: CONTRACT_VERSION,
                    summary: summarize_deterministically(&note.body),
                    confidence: 1.0,
                    warnings: vec![],
                };
                if starter_database::complete_summary_job(
                    &db,
                    *job_id,
                    *note_id,
                    &output.summary,
                    now_ms(),
                )
                .await?
                {
                    log_event("job.summarize_note.completed", &job_id.to_string(), &output);
                }
                message.ack();
            }
        }
    }
    Ok(())
}

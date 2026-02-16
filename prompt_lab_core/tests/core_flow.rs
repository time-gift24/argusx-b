use argusx_common::config::Settings;
use prompt_lab_core::{
    AiExecutionLogFilter, AppendAiExecutionLogInput, BindGoldenSetItemInput, ChecklistFilter,
    ChecklistStatus, CreateChecklistItemInput, ExecStatus, PromptLab, SourceType, TargetLevel,
    UpdateChecklistItemInput, UpsertCheckResultInput,
};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};

static DB_COUNTER: AtomicU64 = AtomicU64::new(0);
fn settings_for_temp() -> Settings {
    let seq = DB_COUNTER.fetch_add(1, Ordering::Relaxed);
    let unique = format!(
        "prompt_lab_test_{}_{}_{}_{}.db",
        std::process::id(),
        std::thread::current().name().unwrap_or("unnamed"),
        seq,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    );
    let path = std::env::temp_dir().join(unique);
    Settings {
        database: argusx_common::config::DatabaseConfig {
            path: path.to_string_lossy().to_string(),
            busy_timeout_ms: 5_000,
            max_connections: 5,
        },
        logging: argusx_common::config::LoggingConfig::default(),
    }
}

#[tokio::test]
async fn checklist_create_update_and_list_roundtrip() {
    let settings = settings_for_temp();
    let lab = PromptLab::new(settings).await.expect("init prompt lab");

    let created = lab
        .checklist_service()
        .create(CreateChecklistItemInput {
            name: "Rule A".to_string(),
            prompt: "Please check step quality".to_string(),
            target_level: TargetLevel::Step,
            result_schema: Some(
                json!({"type": "object", "properties": {"score": {"type": "integer"}}}),
            ),
            version: Some(1),
            status: ChecklistStatus::Active,
            created_by: Some(1001),
        })
        .await
        .expect("create checklist item");

    let updated = lab
        .checklist_service()
        .update(UpdateChecklistItemInput {
            id: created.id,
            name: Some("Rule A+".to_string()),
            prompt: None,
            target_level: None,
            result_schema: None,
            version: Some(2),
            status: Some(ChecklistStatus::Draft),
            updated_by: Some(2002),
        })
        .await
        .expect("update checklist item");

    assert_eq!(updated.name, "Rule A+");
    assert_eq!(updated.version, 2);
    assert_eq!(updated.status, ChecklistStatus::Draft);

    let listed = lab
        .checklist_service()
        .list(ChecklistFilter {
            status: Some(ChecklistStatus::Draft),
            target_level: Some(TargetLevel::Step),
        })
        .await
        .expect("list checklist items");

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, created.id);
}

#[tokio::test]
async fn checklist_create_rejects_invalid_json_schema() {
    let settings = settings_for_temp();
    let lab = PromptLab::new(settings).await.expect("init prompt lab");

    let result = lab
        .checklist_service()
        .create(CreateChecklistItemInput {
            name: "Invalid JSON Rule".to_string(),
            prompt: "check".to_string(),
            target_level: TargetLevel::Step,
            result_schema: Some(json!("{not-json")),
            version: Some(1),
            status: ChecklistStatus::Active,
            created_by: None,
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn golden_set_bind_enforces_foreign_keys() {
    let settings = settings_for_temp();
    let lab = PromptLab::new(settings).await.expect("init prompt lab");

    let checklist = lab
        .checklist_service()
        .create(CreateChecklistItemInput {
            name: "Rule B".to_string(),
            prompt: "check".to_string(),
            target_level: TargetLevel::Step,
            result_schema: Some(json!({"type": "object"})),
            version: Some(1),
            status: ChecklistStatus::Active,
            created_by: None,
        })
        .await
        .expect("create checklist");

    let fk_error = lab
        .golden_set_service()
        .bind(BindGoldenSetItemInput {
            golden_set_id: 999_999,
            checklist_item_id: checklist.id,
            sort_order: 1,
        })
        .await;
    assert!(fk_error.is_err());

    let check_result = lab
        .check_result_service()
        .upsert(UpsertCheckResultInput {
            id: None,
            context_type: "sop".to_string(),
            context_id: 101,
            check_item_id: checklist.id,
            source_type: SourceType::Ai,
            operator_id: Some("system".to_string()),
            result: Some(json!({"ok": true})),
            is_pass: true,
        })
        .await
        .expect("create check result");

    let bound = lab
        .golden_set_service()
        .bind(BindGoldenSetItemInput {
            golden_set_id: check_result.id,
            checklist_item_id: checklist.id,
            sort_order: 2,
        })
        .await
        .expect("bind golden set");

    assert_eq!(bound.golden_set_id, check_result.id);
}

#[tokio::test]
async fn check_run_and_log_append_roundtrip() {
    let settings = settings_for_temp();
    let lab = PromptLab::new(settings).await.expect("init prompt lab");

    let checklist = lab
        .checklist_service()
        .create(CreateChecklistItemInput {
            name: "Rule C".to_string(),
            prompt: "check".to_string(),
            target_level: TargetLevel::Sop,
            result_schema: Some(json!({"type": "object"})),
            version: Some(1),
            status: ChecklistStatus::Active,
            created_by: None,
        })
        .await
        .expect("create checklist");

    let check_result = lab
        .check_result_service()
        .upsert(UpsertCheckResultInput {
            id: None,
            context_type: "sop".to_string(),
            context_id: 202,
            check_item_id: checklist.id,
            source_type: SourceType::Manual,
            operator_id: Some("u-1".to_string()),
            result: Some(json!({"score": 95})),
            is_pass: true,
        })
        .await
        .expect("upsert check result");

    let log = lab
        .ai_log_service()
        .append(AppendAiExecutionLogInput {
            check_result_id: Some(check_result.id),
            context_type: "sop".to_string(),
            context_id: 202,
            check_item_id: checklist.id,
            model_provider: Some("openai".to_string()),
            model_version: "gpt-4o".to_string(),
            temperature: Some(0.2),
            prompt_snapshot: Some("prompt text".to_string()),
            raw_output: Some("{\"score\":95}".to_string()),
            input_tokens: Some(111),
            output_tokens: Some(22),
            exec_status: ExecStatus::Success,
            error_message: None,
            latency_ms: Some(980),
        })
        .await
        .expect("append ai log");

    assert_eq!(log.check_result_id, Some(check_result.id));

    let logs = lab
        .ai_log_service()
        .list(AiExecutionLogFilter {
            context_type: Some("sop".to_string()),
            context_id: Some(202),
            check_item_id: Some(checklist.id),
        })
        .await
        .expect("list logs");

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].id, log.id);
}

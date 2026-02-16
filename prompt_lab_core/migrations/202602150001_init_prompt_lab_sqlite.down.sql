DROP INDEX IF EXISTS idx_rule_analysis;
DROP INDEX IF EXISTS idx_context_log;
DROP TABLE IF EXISTS ai_execution_logs;

DROP INDEX IF EXISTS idx_gsi_item;
DROP TABLE IF EXISTS golden_set_items;

DROP INDEX IF EXISTS idx_rule_history;
DROP INDEX IF EXISTS idx_context_ref;
DROP TABLE IF EXISTS check_results;

DROP TRIGGER IF EXISTS trg_checklist_items_updated_at;
DROP INDEX IF EXISTS idx_checklist_level;
DROP INDEX IF EXISTS idx_checklist_status;
DROP TABLE IF EXISTS checklist_items;

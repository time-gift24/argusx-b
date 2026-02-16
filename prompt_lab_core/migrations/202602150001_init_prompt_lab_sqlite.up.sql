PRAGMA foreign_keys = ON;

CREATE TABLE checklist_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  prompt TEXT NOT NULL,
  target_level TEXT NOT NULL DEFAULT 'step' CHECK (target_level IN ('step', 'sop')),
  result_schema TEXT CHECK (result_schema IS NULL OR json_valid(result_schema)),
  version INTEGER NOT NULL DEFAULT 1,
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'draft')),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  created_by INTEGER,
  updated_by INTEGER,
  deleted_at TEXT
);

CREATE INDEX idx_checklist_status ON checklist_items (status);
CREATE INDEX idx_checklist_level ON checklist_items (target_level);

CREATE TRIGGER trg_checklist_items_updated_at
AFTER UPDATE ON checklist_items
FOR EACH ROW
WHEN NEW.updated_at = OLD.updated_at
BEGIN
  UPDATE checklist_items
  SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
  WHERE id = NEW.id;
END;

CREATE TABLE check_results (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  context_type TEXT NOT NULL,
  context_id INTEGER NOT NULL,
  check_item_id INTEGER NOT NULL,
  source_type INTEGER NOT NULL DEFAULT 1 CHECK (source_type IN (1, 2)),
  operator_id TEXT,
  result TEXT CHECK (result IS NULL OR json_valid(result)),
  is_pass INTEGER NOT NULL DEFAULT 0 CHECK (is_pass IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  FOREIGN KEY (check_item_id) REFERENCES checklist_items(id)
);

CREATE INDEX idx_context_ref ON check_results (context_type, context_id);
CREATE INDEX idx_rule_history ON check_results (check_item_id);

CREATE TABLE golden_set_items (
  golden_set_id INTEGER NOT NULL,
  checklist_item_id INTEGER NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  PRIMARY KEY (golden_set_id, checklist_item_id),
  FOREIGN KEY (golden_set_id) REFERENCES check_results(id),
  FOREIGN KEY (checklist_item_id) REFERENCES checklist_items(id)
);

CREATE INDEX idx_gsi_item ON golden_set_items (checklist_item_id);

CREATE TABLE ai_execution_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  check_result_id INTEGER,
  context_type TEXT NOT NULL,
  context_id INTEGER NOT NULL,
  check_item_id INTEGER NOT NULL,
  model_provider TEXT,
  model_version TEXT NOT NULL,
  temperature REAL DEFAULT 0.0,
  prompt_snapshot TEXT,
  raw_output TEXT,
  input_tokens INTEGER NOT NULL DEFAULT 0,
  output_tokens INTEGER NOT NULL DEFAULT 0,
  exec_status INTEGER NOT NULL DEFAULT 0 CHECK (exec_status IN (0, 1, 2, 3)),
  error_message TEXT,
  latency_ms INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  FOREIGN KEY (check_result_id) REFERENCES check_results(id),
  FOREIGN KEY (check_item_id) REFERENCES checklist_items(id)
);

CREATE INDEX idx_context_log ON ai_execution_logs (context_type, context_id);
CREATE INDEX idx_rule_analysis ON ai_execution_logs (check_item_id, created_at);

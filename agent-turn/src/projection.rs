use agent_core::{RunStreamEvent, UiThreadEvent};
use tokio::sync::mpsc;

pub fn emit_run_events(
    tx: &mpsc::UnboundedSender<RunStreamEvent>,
    events: impl IntoIterator<Item = RunStreamEvent>,
) {
    for event in events {
        let _ = tx.send(event);
    }
}

pub fn emit_ui_events(
    tx: &mpsc::UnboundedSender<UiThreadEvent>,
    events: impl IntoIterator<Item = UiThreadEvent>,
) {
    for event in events {
        let _ = tx.send(event);
    }
}

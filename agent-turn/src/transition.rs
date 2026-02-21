use agent_core::{RunStreamEvent, TranscriptItem, UiThreadEvent};

use crate::effect::Effect;
use crate::state::TurnState;

#[derive(Debug)]
pub struct Transition {
    pub state: TurnState,
    pub new_items: Vec<TranscriptItem>,
    pub run_events: Vec<RunStreamEvent>,
    pub ui_events: Vec<UiThreadEvent>,
    pub effects: Vec<Effect>,
}

impl Transition {
    pub fn new(state: TurnState) -> Self {
        Self {
            state,
            new_items: Vec::new(),
            run_events: Vec::new(),
            ui_events: Vec::new(),
            effects: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: TranscriptItem) {
        self.state.transcript.push(item.clone());
        self.new_items.push(item);
    }

    pub fn add_run_event(&mut self, event: RunStreamEvent) {
        self.run_events.push(event);
    }

    pub fn add_ui_event(&mut self, event: UiThreadEvent) {
        self.ui_events.push(event);
    }

    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
}

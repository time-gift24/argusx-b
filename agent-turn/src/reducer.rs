use agent_core::{
    InputEnvelope, RunStreamEvent, RuntimeEvent, ToolCallStatus, ToolResult, TranscriptItem,
    TurnStats, UiThreadEvent,
};

use crate::effect::Effect;
use crate::state::{Lifecycle, ModelState, TurnEngineConfig, TurnState};
use crate::transition::Transition;

pub fn reduce(state: TurnState, event: RuntimeEvent, config: &TurnEngineConfig) -> Transition {
    let mut tr = Transition::new(state);

    if tr.state.mark_seen(event.id()) {
        return tr;
    }

    if matches!(tr.state.lifecycle, Lifecycle::Done | Lifecycle::Failed) {
        return tr;
    }

    match event {
        RuntimeEvent::TurnStarted { turn_id, input, .. } => {
            if turn_id != tr.state.meta.turn_id {
                tr.add_run_event(RunStreamEvent::ProtocolWarning {
                    turn_id: tr.state.meta.turn_id.clone(),
                    message: format!(
                        "turn id mismatch: expected {}, got {turn_id}",
                        tr.state.meta.turn_id
                    ),
                });
                return tr;
            }

            tr.add_item(TranscriptItem::user_message(input.clone()));
            tr.state.enqueue_input(input);
            tr.add_run_event(RunStreamEvent::TurnStart {
                turn_id: tr.state.meta.turn_id.clone(),
            });
            start_model_from_pending(&mut tr, 0);
        }
        RuntimeEvent::ModelTextDelta { epoch, delta, .. } => {
            if !is_active_epoch(&tr.state, epoch) {
                return tr;
            }
            tr.state.output_buffer.push_str(&delta);
            tr.add_ui_event(UiThreadEvent::MessageDelta {
                turn_id: tr.state.meta.turn_id.clone(),
                delta,
            });
        }
        RuntimeEvent::ModelReasoningDelta { epoch, delta, .. } => {
            if !is_active_epoch(&tr.state, epoch) {
                return tr;
            }
            tr.state.reasoning_buffer.push_str(&delta);
            tr.add_ui_event(UiThreadEvent::ReasoningDelta {
                turn_id: tr.state.meta.turn_id.clone(),
                delta,
            });
        }
        RuntimeEvent::ModelToolCall { epoch, call, .. } => {
            if !is_active_epoch(&tr.state, epoch) {
                return tr;
            }
            let call_id = call.call_id.clone();
            if tr.state.inflight_tools.contains_key(&call_id) {
                tr.add_run_event(RunStreamEvent::ProtocolWarning {
                    turn_id: tr.state.meta.turn_id.clone(),
                    message: format!("duplicate tool call id: {call_id}"),
                });
                return tr;
            }

            tr.state
                .inflight_tools
                .insert(call_id.clone(), call.clone());
            tr.add_item(TranscriptItem::tool_call(epoch, call.clone()));
            tr.add_run_event(RunStreamEvent::ToolExecutionPlanned {
                turn_id: tr.state.meta.turn_id.clone(),
                call_id: call_id.clone(),
                tool_name: call.tool_name.clone(),
            });
            tr.add_ui_event(UiThreadEvent::ToolCallRequested {
                turn_id: tr.state.meta.turn_id.clone(),
                call_id,
                tool_name: call.tool_name.clone(),
            });
            tr.add_effect(Effect::ExecuteTool { epoch, call });
        }
        RuntimeEvent::ToolDispatched { epoch, call_id, .. } => {
            if !is_active_epoch(&tr.state, epoch) {
                return tr;
            }
            if let Some(call) = tr.state.inflight_tools.get(&call_id) {
                tr.add_run_event(RunStreamEvent::ToolExecutionStart {
                    turn_id: tr.state.meta.turn_id.clone(),
                    call_id: call_id.clone(),
                    tool_name: call.tool_name.clone(),
                });
                tr.add_ui_event(UiThreadEvent::ToolCallProgress {
                    turn_id: tr.state.meta.turn_id.clone(),
                    call_id,
                    status: ToolCallStatus::Running,
                });
            }
        }
        RuntimeEvent::ToolResultOk { epoch, result, .. } => {
            on_tool_result(&mut tr, epoch, result, false);
        }
        RuntimeEvent::ToolResultErr { epoch, result, .. } => {
            on_tool_result(&mut tr, epoch, result, true);
        }
        RuntimeEvent::InputInjected { input, .. } => {
            tr.add_item(TranscriptItem::user_message(input.clone()));
            tr.add_run_event(RunStreamEvent::InputInjected {
                turn_id: tr.state.meta.turn_id.clone(),
                input_id: input.id.clone(),
            });
            tr.state.enqueue_input(input);
            if tr.state.model_state == ModelState::Completed && tr.state.inflight_tools.is_empty() {
                let next_epoch = tr.state.epoch.saturating_add(1);
                start_model_from_pending(&mut tr, next_epoch);
            }
        }
        RuntimeEvent::ModelCompleted { epoch, usage, .. } => {
            if !is_active_epoch(&tr.state, epoch) {
                return tr;
            }
            if let Some(usage) = usage.as_ref() {
                tr.state.usage.merge(usage);
            }
            tr.state.model_state = ModelState::Completed;
            tr.state.retry_attempt = 0;

            tr.add_run_event(RunStreamEvent::ModelCompleted {
                turn_id: tr.state.meta.turn_id.clone(),
                usage,
            });

            if tr.state.inflight_tools.is_empty() && !tr.state.pending_inputs.is_empty() {
                let next_epoch = tr.state.epoch.saturating_add(1);
                start_model_from_pending(&mut tr, next_epoch);
            }
        }
        RuntimeEvent::RetryTimerFired { next_epoch, .. } => {
            if tr.state.lifecycle != Lifecycle::Backoff {
                return tr;
            }
            tr.state.lifecycle = Lifecycle::Active;
            tr.state.model_state = ModelState::Streaming;
            tr.state.epoch = next_epoch;

            let inputs = if tr.state.pending_inputs.is_empty() {
                tr.state.last_request_inputs.clone()
            } else {
                tr.state.drain_pending_inputs()
            };

            if inputs.is_empty() {
                let message = "retry fired but no input available".to_string();
                fail_turn(&mut tr, message);
                return tr;
            }

            tr.state.last_request_inputs = inputs.clone();
            tr.add_effect(Effect::StartModel {
                epoch: next_epoch,
                transcript: tr.state.transcript.clone(),
                inputs,
            });
        }
        RuntimeEvent::TransientError {
            epoch,
            message,
            retry_after_ms,
            ..
        } => {
            if epoch != tr.state.epoch {
                return tr;
            }
            tr.state.model_state = ModelState::Error;
            let attempt = tr.state.retry_attempt.saturating_add(1);
            let can_retry = attempt <= config.retry_policy.max_retries;
            tr.add_run_event(RunStreamEvent::TransientError {
                turn_id: tr.state.meta.turn_id.clone(),
                message: message.clone(),
                can_retry,
            });

            if can_retry {
                tr.state.retry_attempt = attempt;
                tr.state.lifecycle = Lifecycle::Backoff;
                let delay_ms = retry_after_ms.unwrap_or_else(|| backoff_ms(config, attempt));
                let next_epoch = tr.state.epoch.saturating_add(1);

                tr.add_run_event(RunStreamEvent::Retrying {
                    turn_id: tr.state.meta.turn_id.clone(),
                    attempt,
                    next_epoch,
                    delay_ms,
                });
                tr.add_ui_event(UiThreadEvent::Warning {
                    turn_id: tr.state.meta.turn_id.clone(),
                    message,
                });
                tr.add_effect(Effect::ScheduleRetry {
                    delay_ms,
                    next_epoch,
                });
            } else {
                fail_turn(&mut tr, message);
            }
        }
        RuntimeEvent::FatalError { message, .. } => {
            fail_turn(&mut tr, message);
        }
        RuntimeEvent::CancelRequested { reason, .. } => {
            let message = reason.unwrap_or_else(|| "turn cancelled".to_string());
            fail_turn(&mut tr, message);
            tr.add_effect(Effect::CancelInflightTools);
        }
    }

    maybe_finalize(&mut tr);
    tr
}

fn is_active_epoch(state: &TurnState, epoch: u64) -> bool {
    state.lifecycle == Lifecycle::Active
        && state.model_state == ModelState::Streaming
        && epoch == state.epoch
}

fn start_model_from_pending(tr: &mut Transition, next_epoch: u64) {
    let inputs = tr.state.drain_pending_inputs();
    if inputs.is_empty() {
        return;
    }

    tr.state.epoch = next_epoch;
    tr.state.lifecycle = Lifecycle::Active;
    tr.state.model_state = ModelState::Streaming;
    tr.state.last_request_inputs = inputs.clone();
    tr.add_effect(Effect::StartModel {
        epoch: next_epoch,
        transcript: tr.state.transcript.clone(),
        inputs,
    });
}

fn on_tool_result(tr: &mut Transition, epoch: u64, result: ToolResult, is_error: bool) {
    if epoch != tr.state.epoch {
        return;
    }
    let call_id = result.call_id.clone();
    if tr.state.inflight_tools.remove(&call_id).is_none() {
        tr.add_run_event(RunStreamEvent::ProtocolWarning {
            turn_id: tr.state.meta.turn_id.clone(),
            message: format!("tool result without inflight call: {call_id}"),
        });
        return;
    }

    tr.add_item(TranscriptItem::tool_result(epoch, result.clone()));

    if is_error {
        tr.add_run_event(RunStreamEvent::ToolExecutionError {
            turn_id: tr.state.meta.turn_id.clone(),
            result: result.clone(),
        });
        tr.add_ui_event(UiThreadEvent::ToolCallProgress {
            turn_id: tr.state.meta.turn_id.clone(),
            call_id: call_id.clone(),
            status: ToolCallStatus::Failed,
        });
    } else {
        tr.add_run_event(RunStreamEvent::ToolExecutionDone {
            turn_id: tr.state.meta.turn_id.clone(),
            result: result.clone(),
        });
        tr.add_ui_event(UiThreadEvent::ToolCallProgress {
            turn_id: tr.state.meta.turn_id.clone(),
            call_id: call_id.clone(),
            status: ToolCallStatus::Completed,
        });
    }

    tr.add_ui_event(UiThreadEvent::ToolCallCompleted {
        turn_id: tr.state.meta.turn_id.clone(),
        result: result.clone(),
    });

    tr.state
        .enqueue_input(InputEnvelope::tool_json(result.output));

    if tr.state.model_state == ModelState::Completed && tr.state.inflight_tools.is_empty() {
        let next_epoch = tr.state.epoch.saturating_add(1);
        start_model_from_pending(tr, next_epoch);
    }
}

fn maybe_finalize(tr: &mut Transition) {
    if !tr.state.can_finish() {
        return;
    }

    if !tr.state.reasoning_buffer.is_empty() {
        tr.add_item(TranscriptItem::reasoning(tr.state.reasoning_buffer.clone()));
    }

    if !tr.state.output_buffer.is_empty() {
        tr.add_item(TranscriptItem::assistant_message(
            tr.state.output_buffer.clone(),
        ));
    }

    tr.state.done_emitted = true;
    tr.state.lifecycle = Lifecycle::Done;

    let stats = TurnStats {
        tool_calls_count: tr.state.tool_calls_count(),
        total_input_tokens: tr.state.usage.input_tokens,
        total_output_tokens: tr.state.usage.output_tokens,
    };

    let final_message =
        (!tr.state.output_buffer.is_empty()).then_some(tr.state.output_buffer.clone());

    tr.add_run_event(RunStreamEvent::TurnDone {
        turn_id: tr.state.meta.turn_id.clone(),
        final_message: final_message.clone(),
        usage: tr.state.usage.clone(),
        stats: stats.clone(),
    });

    tr.add_ui_event(UiThreadEvent::Done {
        turn_id: tr.state.meta.turn_id.clone(),
        summary: final_message,
        stats,
    });

    tr.add_effect(Effect::PersistCheckpoint);
}

fn fail_turn(tr: &mut Transition, message: String) {
    tr.state.lifecycle = Lifecycle::Failed;
    tr.state.model_state = ModelState::Error;
    tr.add_item(TranscriptItem::system_note(
        agent_core::NoteLevel::Error,
        message.clone(),
    ));
    tr.add_run_event(RunStreamEvent::TurnFailed {
        turn_id: tr.state.meta.turn_id.clone(),
        message: message.clone(),
    });
    tr.add_ui_event(UiThreadEvent::Error {
        turn_id: tr.state.meta.turn_id.clone(),
        message,
    });
    tr.add_effect(Effect::PersistCheckpoint);
}

fn backoff_ms(config: &TurnEngineConfig, attempt: u32) -> u64 {
    let capped = attempt.saturating_sub(1).min(10);
    let multiplier = 1_u64.checked_shl(capped).unwrap_or(u64::MAX);
    config.retry_policy.base_delay_ms.saturating_mul(multiplier)
}

#[cfg(test)]
mod tests {
    use agent_core::{InputEnvelope, RuntimeEvent, SessionMeta, ToolCall, Usage};

    use super::*;

    fn base_state() -> TurnState {
        TurnState::new(SessionMeta::new("s1", "t1"))
    }

    fn cfg() -> TurnEngineConfig {
        TurnEngineConfig::default()
    }

    #[test]
    fn replay_is_deterministic() {
        let run_once = || {
            let mut state = base_state();
            let mut runs = Vec::new();

            let events = vec![
                RuntimeEvent::TurnStarted {
                    event_id: "e1".into(),
                    turn_id: "t1".into(),
                    input: InputEnvelope::user_text("hi"),
                },
                RuntimeEvent::ModelTextDelta {
                    event_id: "e2".into(),
                    epoch: 0,
                    delta: "hello".into(),
                },
                RuntimeEvent::ModelCompleted {
                    event_id: "e3".into(),
                    epoch: 0,
                    usage: Some(Usage {
                        input_tokens: 1,
                        output_tokens: 1,
                        total_tokens: 2,
                    }),
                },
            ];

            for ev in events {
                let tr = reduce(state, ev, &cfg());
                runs.extend(tr.run_events.clone());
                state = tr.state;
            }
            (
                state.lifecycle,
                state.done_emitted,
                state.transcript.len(),
                runs,
            )
        };

        let a = run_once();
        let b = run_once();
        assert_eq!(a, b);
    }

    #[test]
    fn completed_does_not_finish_when_tool_inflight() {
        let mut state = base_state();

        state = reduce(
            state,
            RuntimeEvent::TurnStarted {
                event_id: "e1".into(),
                turn_id: "t1".into(),
                input: InputEnvelope::user_text("hi"),
            },
            &cfg(),
        )
        .state;

        state = reduce(
            state,
            RuntimeEvent::ModelToolCall {
                event_id: "e2".into(),
                epoch: 0,
                call: ToolCall {
                    call_id: "c1".into(),
                    tool_name: "echo".into(),
                    arguments: serde_json::json!({}),
                },
            },
            &cfg(),
        )
        .state;

        state = reduce(
            state,
            RuntimeEvent::ModelCompleted {
                event_id: "e3".into(),
                epoch: 0,
                usage: None,
            },
            &cfg(),
        )
        .state;

        assert_eq!(state.model_state, ModelState::Completed);
        assert!(!state.done_emitted);
    }

    #[test]
    fn live_injection_after_completed_restarts_model() {
        let mut state = base_state();

        state = reduce(
            state,
            RuntimeEvent::TurnStarted {
                event_id: "e1".into(),
                turn_id: "t1".into(),
                input: InputEnvelope::user_text("first"),
            },
            &cfg(),
        )
        .state;

        state = reduce(
            state,
            RuntimeEvent::InputInjected {
                event_id: "e2".into(),
                input: InputEnvelope::user_text("second"),
            },
            &cfg(),
        )
        .state;

        let tr = reduce(
            state,
            RuntimeEvent::ModelCompleted {
                event_id: "e3".into(),
                epoch: 0,
                usage: None,
            },
            &cfg(),
        );

        assert_eq!(tr.state.epoch, 1);
        assert_eq!(tr.state.model_state, ModelState::Streaming);
        assert!(tr
            .effects
            .iter()
            .any(|e| matches!(e, Effect::StartModel { epoch: 1, .. })));
    }

    #[test]
    fn transient_error_schedules_retry_and_bumps_epoch() {
        let mut state = base_state();
        state = reduce(
            state,
            RuntimeEvent::TurnStarted {
                event_id: "e1".into(),
                turn_id: "t1".into(),
                input: InputEnvelope::user_text("hi"),
            },
            &cfg(),
        )
        .state;

        let backoff = reduce(
            state,
            RuntimeEvent::TransientError {
                event_id: "e2".into(),
                epoch: 0,
                message: "timeout".into(),
                retry_after_ms: Some(1),
            },
            &cfg(),
        );

        assert_eq!(backoff.state.lifecycle, Lifecycle::Backoff);
        assert!(backoff
            .effects
            .iter()
            .any(|e| matches!(e, Effect::ScheduleRetry { next_epoch: 1, .. })));

        let retry = reduce(
            backoff.state,
            RuntimeEvent::RetryTimerFired {
                event_id: "e3".into(),
                next_epoch: 1,
            },
            &cfg(),
        );

        assert_eq!(retry.state.lifecycle, Lifecycle::Active);
        assert_eq!(retry.state.epoch, 1);
        assert!(retry
            .effects
            .iter()
            .any(|e| matches!(e, Effect::StartModel { epoch: 1, .. })));
    }

    #[test]
    fn turn_done_is_emitted_once() {
        let mut state = base_state();
        state = reduce(
            state,
            RuntimeEvent::TurnStarted {
                event_id: "e1".into(),
                turn_id: "t1".into(),
                input: InputEnvelope::user_text("hi"),
            },
            &cfg(),
        )
        .state;

        state = reduce(
            state,
            RuntimeEvent::ModelTextDelta {
                event_id: "e2".into(),
                epoch: 0,
                delta: "done".into(),
            },
            &cfg(),
        )
        .state;

        let first = reduce(
            state,
            RuntimeEvent::ModelCompleted {
                event_id: "e3".into(),
                epoch: 0,
                usage: None,
            },
            &cfg(),
        );

        assert!(first
            .run_events
            .iter()
            .any(|e| matches!(e, RunStreamEvent::TurnDone { .. })));

        let second = reduce(
            first.state,
            RuntimeEvent::ModelCompleted {
                event_id: "e4".into(),
                epoch: 0,
                usage: None,
            },
            &cfg(),
        );

        assert!(!second
            .run_events
            .iter()
            .any(|e| matches!(e, RunStreamEvent::TurnDone { .. })));
    }
}

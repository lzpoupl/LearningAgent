//! 推送抽象；唯一接触 Tauri `Channel` 的服务层文件，便于循环逻辑脱离 Tauri 测试。

use std::sync::{Arc, Mutex};

use tauri::ipc::Channel;

use crate::interface::event::AgentEvent;

/// 事件发射器；循环只依赖该抽象。
pub trait EventEmitter: Send + Sync {
    fn emit(&self, event: AgentEvent);
}

/// 基于 Tauri Channel 的实现；`Channel` 内部是 `Arc`，克隆后指向同一通道。
#[derive(Clone)]
pub struct ChannelEmitter {
    channel: Channel<AgentEvent>,
}

impl ChannelEmitter {
    pub fn new(channel: Channel<AgentEvent>) -> Self {
        Self { channel }
    }
}

impl EventEmitter for ChannelEmitter {
    fn emit(&self, event: AgentEvent) {
        if let Err(error) = self.channel.send(event) {
            // 发送失败（窗口已关闭等）只记日志，不中断循环。
            eprintln!("事件推送失败: {error}");
        }
    }
}

/// 记录型 emitter；测试用它断言事件序列。
#[allow(dead_code)]
#[derive(Default)]
pub struct RecordingEmitter {
    events: Mutex<Vec<AgentEvent>>,
}

#[allow(dead_code)]
impl RecordingEmitter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<AgentEvent> {
        self.events.lock().expect("事件记录锁已中毒").clone()
    }

    pub fn into_arc(self) -> Arc<dyn EventEmitter> {
        Arc::new(self)
    }
}

impl EventEmitter for RecordingEmitter {
    fn emit(&self, event: AgentEvent) {
        self.events.lock().expect("事件记录锁已中毒").push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::session::TurnStatus;

    #[test]
    fn recording_emitter_keeps_event_order() {
        let emitter = RecordingEmitter::new();
        emitter.emit(AgentEvent::TurnStarted {
            session_id: 1,
            turn_id: "t-1".to_string(),
        });
        emitter.emit(AgentEvent::TurnEnded {
            session_id: 1,
            turn_id: "t-1".to_string(),
            status: TurnStatus::Completed,
            error: None,
        });

        let events = emitter.events();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], AgentEvent::TurnStarted { .. }));
        assert!(matches!(
            events[1],
            AgentEvent::TurnEnded {
                status: TurnStatus::Completed,
                ..
            }
        ));
    }
}

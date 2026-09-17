//! 轮次注册表与取消 / 裁决输入通道。

use std::collections::HashMap;
use std::sync::Mutex;

use tokio::sync::{mpsc, watch};

use crate::interface::error::ApiError;
use crate::interface::session::ApprovalDecision;

/// 循环侧等待的外部输入。
#[derive(Clone, Debug)]
pub enum TurnCommand {
    /// 对 `ask` 工具调用的裁决。
    Approve {
        call_id: String,
        decision: ApprovalDecision,
    },
}

struct TurnEntry {
    turn_id: String,
    cancel: watch::Sender<bool>,
    commands: mpsc::UnboundedSender<TurnCommand>,
}

/// 循环持有的控制句柄：取消信号与外部输入。
#[derive(Debug)]
pub struct TurnControlHandle {
    cancel: watch::Receiver<bool>,
    commands: mpsc::UnboundedReceiver<TurnCommand>,
}

impl TurnControlHandle {
    pub fn is_cancelled(&self) -> bool {
        *self.cancel.borrow()
    }

    /// 拆出两个接收端，便于在 `select!` 中同时等待取消与裁决。
    pub fn split(
        &mut self,
    ) -> (
        &mut watch::Receiver<bool>,
        &mut mpsc::UnboundedReceiver<TurnCommand>,
    ) {
        (&mut self.cancel, &mut self.commands)
    }

    /// 等待取消信号；已取消时立即返回。
    pub async fn wait_cancel(&mut self) {
        let _ = self.cancel.changed().await;
    }
}

/// 轮次注册表；每个会话同一时刻至多一个进行中的轮次。
#[derive(Default)]
pub struct TurnRegistry {
    active: Mutex<HashMap<i64, TurnEntry>>,
}

impl TurnRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 受理一轮；该会话已有进行中的轮次时返回 `turn_conflict`。
    pub fn begin(&self, session_id: i64, turn_id: &str) -> Result<TurnControlHandle, ApiError> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| ApiError::internal("轮次注册表不可用"))?;
        if let Some(entry) = active.get(&session_id) {
            return Err(ApiError::turn_conflict(format!(
                "会话 {session_id} 已有进行中的轮次: {}",
                entry.turn_id
            )));
        }

        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        active.insert(
            session_id,
            TurnEntry {
                turn_id: turn_id.to_string(),
                cancel: cancel_tx,
                commands: command_tx,
            },
        );

        Ok(TurnControlHandle {
            cancel: cancel_rx,
            commands: command_rx,
        })
    }

    /// 结束一轮；`turn_id` 不匹配时不影响后来注册的轮次。
    pub fn finish(&self, session_id: i64, turn_id: &str) {
        if let Ok(mut active) = self.active.lock() {
            if active.get(&session_id).map(|entry| entry.turn_id.as_str()) == Some(turn_id) {
                active.remove(&session_id);
            }
        }
    }

    /// 取消进行中的轮次；不存在时返回 `not_found`。
    pub fn cancel(&self, session_id: i64) -> Result<(), ApiError> {
        let active = self
            .active
            .lock()
            .map_err(|_| ApiError::internal("轮次注册表不可用"))?;
        let entry = active
            .get(&session_id)
            .ok_or_else(|| ApiError::not_found(format!("会话没有进行中的轮次: {session_id}")))?;
        entry.cancel.send_replace(true);
        Ok(())
    }

    /// 提交 `ask` 工具调用的裁决；轮次或调用 id 不存在时返回 `not_found`。
    pub fn approve(
        &self,
        session_id: i64,
        turn_id: &str,
        call_id: &str,
        decision: ApprovalDecision,
    ) -> Result<(), ApiError> {
        let active = self
            .active
            .lock()
            .map_err(|_| ApiError::internal("轮次注册表不可用"))?;
        let entry = active
            .get(&session_id)
            .ok_or_else(|| ApiError::not_found(format!("会话没有进行中的轮次: {session_id}")))?;
        if entry.turn_id != turn_id {
            return Err(ApiError::not_found(format!("轮次不匹配: {turn_id}")));
        }

        entry
            .commands
            .send(TurnCommand::Approve {
                call_id: call_id.to_string(),
                decision,
            })
            .map_err(|_| ApiError::not_found(format!("工具调用已结束: {call_id}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_turns_are_rejected() {
        let registry = TurnRegistry::new();
        let _handle = registry.begin(1, "t-1").unwrap();

        assert_eq!(registry.begin(1, "t-2").unwrap_err().code, "turn_conflict");
        // 其它会话互不影响。
        let _other = registry.begin(2, "t-1").unwrap();

        registry.finish(1, "t-1");
        assert!(registry.begin(1, "t-3").is_ok());
    }

    #[test]
    fn cancel_sets_the_watch_signal() {
        let registry = TurnRegistry::new();
        let mut handle = registry.begin(1, "t-1").unwrap();
        assert!(!handle.is_cancelled());

        registry.cancel(1).unwrap();
        let (cancel, _) = handle.split();
        assert!(*cancel.borrow_and_update());

        assert_eq!(registry.cancel(2).unwrap_err().code, "not_found");
    }

    #[tokio::test]
    async fn approve_reaches_the_loop() {
        let registry = TurnRegistry::new();
        let mut handle = registry.begin(1, "t-1").unwrap();

        registry
            .approve(1, "t-1", "call-1", ApprovalDecision::AllowAlways)
            .unwrap();

        let (_, commands) = handle.split();
        match commands.recv().await.unwrap() {
            TurnCommand::Approve { call_id, decision } => {
                assert_eq!(call_id, "call-1");
                assert_eq!(decision, ApprovalDecision::AllowAlways);
            }
        }

        assert_eq!(
            registry
                .approve(1, "t-x", "call-1", ApprovalDecision::Deny)
                .unwrap_err()
                .code,
            "not_found"
        );
        assert_eq!(
            registry
                .approve(9, "t-1", "call-1", ApprovalDecision::Deny)
                .unwrap_err()
                .code,
            "not_found"
        );
    }
}

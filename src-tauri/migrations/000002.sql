-- 卡片调度字段：调度算法与算法私有状态。
ALTER TABLE card ADD COLUMN algorithm TEXT NOT NULL DEFAULT 'sm2';
ALTER TABLE card ADD COLUMN scheduler_state TEXT;

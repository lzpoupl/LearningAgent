-- Agent 会话存储：session、message 两张表。
-- 本文件只在 user_version 从 3 推进到 4 时执行一次，无需幂等。

-- 会话：一个会话属于一个 Agent；消息随会话级联删除。
-- last_provider / last_model 记录最近一轮实际调用的模型，未调用过为 NULL。
CREATE TABLE session (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id      INTEGER NOT NULL,
    title         TEXT    NOT NULL,
    last_provider TEXT,
    last_model    TEXT,
    created_at    TEXT    NOT NULL,
    updated_at    TEXT    NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agent (id) ON DELETE CASCADE
);

CREATE INDEX idx_session_agent ON session (agent_id, updated_at DESC);

-- 消息：系统提示词快照、用户输入、助手回答与工具调用、工具结果。
CREATE TABLE message (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id        INTEGER NOT NULL,
    turn_id           TEXT,                              -- 同一轮的 user/assistant/tool 共用
    role              TEXT    NOT NULL
                      CHECK (role IN ('system', 'user', 'assistant', 'tool')),
    content           TEXT    NOT NULL DEFAULT '',
    tool_calls        TEXT,                              -- JSON 数组，仅 assistant 使用
    tool_call_id      TEXT,                              -- 仅 tool 使用
    tool_name         TEXT,                              -- 仅 tool 使用，<group>.<id>
    status            TEXT    NOT NULL DEFAULT 'complete'
                      CHECK (status IN ('streaming', 'complete', 'error', 'interrupted')),
    prompt_tokens     INTEGER,
    completion_tokens INTEGER,
    created_at        TEXT    NOT NULL,
    updated_at        TEXT    NOT NULL,
    FOREIGN KEY (session_id) REFERENCES session (id) ON DELETE CASCADE
);

CREATE INDEX idx_message_session ON message (session_id, id);

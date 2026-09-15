-- Agent 模块基础表结构：Agent、工具组、工具权限与 bucket。
-- 本文件只在 user_version 从 1 推进到 2 时执行一次，无需幂等。

-- Agent 定义。
CREATE TABLE agent (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT    NOT NULL UNIQUE,
    description   TEXT    NOT NULL DEFAULT '',
    icon          TEXT    NOT NULL DEFAULT 'AI',
    color         TEXT    NOT NULL DEFAULT '',
    system_prompt TEXT    NOT NULL DEFAULT '',
    builtin       INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT    NOT NULL,
    updated_at    TEXT    NOT NULL
);

-- 工具目录：以 (工具组, id) 标识，引用写成 <group_name>.<id>。
CREATE TABLE tool (
    group_name         TEXT    NOT NULL,                -- 工具组，如 anki / asset / user
    id                 TEXT    NOT NULL,                -- 组内 id，如 add_card
    name               TEXT    NOT NULL,
    description        TEXT    NOT NULL,
    parameters         TEXT    NOT NULL DEFAULT '{}',   -- JSON Schema
    returns            TEXT    NOT NULL DEFAULT '{}',   -- JSON Schema
    default_permission TEXT    NOT NULL DEFAULT 'ask'
                       CHECK (default_permission IN ('allow', 'ask', 'deny')),
    builtin            INTEGER NOT NULL DEFAULT 1,
    created_at         TEXT    NOT NULL,
    updated_at         TEXT    NOT NULL,
    PRIMARY KEY (group_name, id)
);

-- Agent 对工具的权限：允许 / 询问 / 拒绝。
CREATE TABLE agent_tool (
    agent_id   INTEGER NOT NULL,
    tool_group TEXT    NOT NULL,
    tool_id    TEXT    NOT NULL,
    permission TEXT    NOT NULL DEFAULT 'ask'
               CHECK (permission IN ('allow', 'ask', 'deny')),
    created_at TEXT    NOT NULL,
    updated_at TEXT    NOT NULL,
    PRIMARY KEY (agent_id, tool_group, tool_id),
    FOREIGN KEY (agent_id) REFERENCES agent (id) ON DELETE CASCADE,
    FOREIGN KEY (tool_group, tool_id) REFERENCES tool (group_name, id) ON DELETE CASCADE
);

CREATE INDEX idx_agent_tool_agent ON agent_tool (agent_id);

-- 非结构化资产 bucket：名称 -> 文件系统目录。root_path 按平台原生形式存储。
CREATE TABLE bucket (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT    NOT NULL UNIQUE,
    root_path  TEXT    NOT NULL,
    created_at TEXT    NOT NULL,
    updated_at TEXT    NOT NULL
);

-- 内置 Agent；id 固定，便于默认授权与前端引用。
INSERT INTO agent
  (id, name, description, icon, color, system_prompt, builtin, created_at, updated_at)
VALUES
  (1, '数学 Agent', '数学问题、公式推导与解题思路',
   '∑', 'linear-gradient(135deg, #438fff, #5c6df5)',
   '你是数学学习助手，负责题目解析、知识点拆解与错题复盘。',
   1, datetime('now'), datetime('now')),
  (2, '英语 Agent', '英语词汇、语法、翻译与口语练习',
   'En', 'linear-gradient(135deg, #26b7bf, #16a5a8)',
   '你是英语学习助手，负责词汇讲解、句子分析与翻译训练。',
   1, datetime('now'), datetime('now'));

-- 内置工具目录（group_name, id, 名称, 描述, 参数, 返回, 默认权限）。
INSERT INTO tool
  (group_name, id, name, description, parameters, returns, default_permission, builtin, created_at, updated_at)
VALUES
  ('anki',  'list_decks',   '列出牌组', '列出指定牌组下的子牌组。',        '{}', '{}', 'allow', 1, datetime('now'), datetime('now')),
  ('anki',  'list_cards',   '列出卡片', '列出牌组下的卡片，可按条件过滤。', '{}', '{}', 'allow', 1, datetime('now'), datetime('now')),
  ('anki',  'get_card',     '读取卡片', '按 id 读取单张卡片。',            '{}', '{}', 'allow', 1, datetime('now'), datetime('now')),
  ('anki',  'search_cards', '搜索卡片', '按关键字搜索卡片。',              '{}', '{}', 'allow', 1, datetime('now'), datetime('now')),
  ('anki',  'add_deck',     '新增牌组', '创建牌组。',                      '{}', '{}', 'ask',   1, datetime('now'), datetime('now')),
  ('anki',  'add_card',     '新增卡片', '向牌组新增卡片。',                '{}', '{}', 'ask',   1, datetime('now'), datetime('now')),
  ('anki',  'update_card',  '修改卡片', '修改卡片正面或背面。',            '{}', '{}', 'ask',   1, datetime('now'), datetime('now')),
  ('anki',  'move_card',    '移动卡片', '把卡片移动到目标牌组。',          '{}', '{}', 'ask',   1, datetime('now'), datetime('now')),
  ('anki',  'grade_card',   '作答卡片', '记录一次作答并推进复习计划。',    '{}', '{}', 'ask',   1, datetime('now'), datetime('now')),
  ('anki',  'delete_card',  '删除卡片', '删除单张卡片。',                  '{}', '{}', 'ask',   1, datetime('now'), datetime('now')),
  ('asset', 'list',         '列出资产', '列出可访问的非结构化资产。',      '{}', '{}', 'allow', 1, datetime('now'), datetime('now')),
  ('asset', 'search',       '检索资产', '在知识库中检索资产内容。',        '{}', '{}', 'allow', 1, datetime('now'), datetime('now')),
  ('asset', 'read',         '读取资产', '读取资产文本内容。',              '{}', '{}', 'allow', 1, datetime('now'), datetime('now')),
  ('user',  'ask_question', '反问用户', '向用户提出澄清问题并等待回答。',  '{}', '{}', 'allow', 1, datetime('now'), datetime('now'));

-- 内置 Agent 的初始工具权限：直接采用各工具的默认级别。
INSERT INTO agent_tool (agent_id, tool_group, tool_id, permission, created_at, updated_at)
SELECT a.id, t.group_name, t.id, t.default_permission, datetime('now'), datetime('now')
FROM agent a JOIN tool t
WHERE a.builtin = 1;

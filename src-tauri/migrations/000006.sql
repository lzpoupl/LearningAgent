-- user 工具契约：补齐 user.ask_question 的 parameters / returns。
-- 本文件只在 user_version 从 5 推进到 6 时执行一次，无需幂等。
-- 顶层只保留 type / properties / required / description；属性级的 items / description
-- 会被 context::normalize_parameters 与 edgee::to_json_schema 原样保留并送达模型。

UPDATE tool SET
  description = '向用户提出一个澄清问题并等待回答；可附带候选答案供用户点选。',
  parameters = '{"type":"object","description":"向用户提出一个澄清问题并等待回答；可附带候选答案供用户点选。","properties":{"question":{"type":"string","description":"需要向用户澄清的问题。"},"options":{"type":"array","items":{"type":"string"},"description":"可选的候选答案，前端渲染为快捷按钮；用户仍可自由输入。最多 6 项。"}},"required":["question"]}',
  returns = '{"type":"object","properties":{"question":{"type":"string"},"answer":{"type":"string"}}}',
  updated_at = datetime('now')
WHERE group_name = 'user' AND id = 'ask_question';

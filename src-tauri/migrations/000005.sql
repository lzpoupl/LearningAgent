-- Anki 工具契约：补齐 parameters / returns JSON Schema 与更明确的描述。
-- 本文件只在 user_version 从 4 推进到 5 时执行一次，无需幂等。
-- 顶层只保留 type / properties / required / description，与 context::normalize_parameters
-- 支持的子集一致；属性级的 enum / description / type 会被原样送达模型。

UPDATE tool SET
  description = '列出指定牌组下的直接子牌组。',
  parameters = '{"type":"object","description":"列出指定牌组下的直接子牌组。","properties":{"deckPath":{"type":"string","description":"父牌组路径，以 / 开头；省略时使用根路径 /。"}}}',
  returns = '{"type":"object","properties":{"decks":{"type":"array","items":{"type":"object"}},"count":{"type":"integer"}}}',
  updated_at = datetime('now')
WHERE group_name = 'anki' AND id = 'list_decks';

UPDATE tool SET
  description = '列出牌组下的卡片，可按状态、关键字与到期时间过滤。',
  parameters = '{"type":"object","description":"列出牌组下的卡片，可按状态、关键字与到期时间过滤。","properties":{"deckPath":{"type":"string","description":"牌组路径，以 / 开头。"},"state":{"type":"string","enum":["new","learning","review","relearning"],"description":"只返回指定记忆状态的卡片。"},"keyword":{"type":"string","description":"在卡片正面或背面进行模糊匹配。"},"dueOnly":{"type":"boolean","description":"为 true 时只返回已到期的卡片；没有到期时间的新卡不会被返回。"},"limit":{"type":"integer","description":"返回数量上限，缺省 50，最大 200。"},"offset":{"type":"integer","description":"跳过前若干条，用于翻页，缺省 0。"}},"required":["deckPath"]}',
  returns = '{"type":"object","properties":{"cards":{"type":"array","items":{"type":"object"}},"count":{"type":"integer"},"hasMore":{"type":"boolean"}}}',
  updated_at = datetime('now')
WHERE group_name = 'anki' AND id = 'list_cards';

UPDATE tool SET
  description = '按 id 读取单张卡片。',
  parameters = '{"type":"object","description":"按 id 读取单张卡片。","properties":{"cardId":{"type":"string","description":"卡片 id，字符串或整数均可。"}},"required":["cardId"]}',
  returns = '{"type":"object","properties":{"card":{"type":"object"}}}',
  updated_at = datetime('now')
WHERE group_name = 'anki' AND id = 'get_card';

UPDATE tool SET
  description = '按关键字搜索卡片，可限定牌组。',
  parameters = '{"type":"object","description":"按关键字搜索卡片，可限定牌组。","properties":{"keyword":{"type":"string","description":"在卡片正面或背面进行模糊匹配。"},"deckPath":{"type":"string","description":"限定搜索范围的牌组路径，省略时跨全部牌组。"},"limit":{"type":"integer","description":"返回数量上限，缺省 50，最大 200。"},"offset":{"type":"integer","description":"跳过前若干条，用于翻页，缺省 0。"}},"required":["keyword"]}',
  returns = '{"type":"object","properties":{"cards":{"type":"array","items":{"type":"object"}},"count":{"type":"integer"},"hasMore":{"type":"boolean"}}}',
  updated_at = datetime('now')
WHERE group_name = 'anki' AND id = 'search_cards';

UPDATE tool SET
  description = '创建牌组，父级路径缺失时自动创建。',
  parameters = '{"type":"object","description":"创建牌组，父级路径缺失时自动创建。","properties":{"deckPath":{"type":"string","description":"要创建的牌组路径，以 / 开头。"}},"required":["deckPath"]}',
  returns = '{"type":"object","properties":{"deckPath":{"type":"string"}}}',
  updated_at = datetime('now')
WHERE group_name = 'anki' AND id = 'add_deck';

UPDATE tool SET
  description = '向指定牌组新增一张卡片。',
  parameters = '{"type":"object","description":"向指定牌组新增一张卡片。","properties":{"deckPath":{"type":"string","description":"目标牌组路径，必须是具体牌组，不能是根 /。"},"front":{"type":"string","description":"卡片正面内容。"},"back":{"type":"string","description":"卡片背面内容。"}},"required":["deckPath","front","back"]}',
  returns = '{"type":"object","properties":{"cardId":{"type":"string"}}}',
  updated_at = datetime('now')
WHERE group_name = 'anki' AND id = 'add_card';

UPDATE tool SET
  description = '修改卡片正面或背面内容，至少提供一项。',
  parameters = '{"type":"object","description":"修改卡片正面或背面内容，至少提供一项。","properties":{"cardId":{"type":"string","description":"卡片 id，字符串或整数均可。"},"front":{"type":"string","description":"新的正面内容，省略时保持不变。"},"back":{"type":"string","description":"新的背面内容，省略时保持不变。"}},"required":["cardId"]}',
  returns = '{"type":"object","properties":{"cardId":{"type":"string"}}}',
  updated_at = datetime('now')
WHERE group_name = 'anki' AND id = 'update_card';

UPDATE tool SET
  description = '把卡片移动到目标牌组。',
  parameters = '{"type":"object","description":"把卡片移动到目标牌组。","properties":{"cardId":{"type":"string","description":"卡片 id，字符串或整数均可。"},"targetDeckPath":{"type":"string","description":"目标牌组路径，必须是具体牌组，不能是根 /。"}},"required":["cardId","targetDeckPath"]}',
  returns = '{"type":"object","properties":{"cardId":{"type":"string"},"deckPath":{"type":"string"}}}',
  updated_at = datetime('now')
WHERE group_name = 'anki' AND id = 'move_card';

UPDATE tool SET
  description = '记录一次作答并推进复习计划。',
  parameters = '{"type":"object","description":"记录一次作答并推进复习计划。","properties":{"cardId":{"type":"string","description":"卡片 id，字符串或整数均可。"},"grade":{"type":"string","enum":["again","hard","good","easy"],"description":"作答等级：重来 / 困难 / 良好 / 简单。"}},"required":["cardId","grade"]}',
  returns = '{"type":"object","properties":{"cardId":{"type":"string"},"state":{"type":"string"},"dueAt":{"type":"string"}}}',
  updated_at = datetime('now')
WHERE group_name = 'anki' AND id = 'grade_card';

UPDATE tool SET
  description = '删除单张卡片。',
  parameters = '{"type":"object","description":"删除单张卡片。","properties":{"cardId":{"type":"string","description":"卡片 id，字符串或整数均可。"}},"required":["cardId"]}',
  returns = '{"type":"object","properties":{"cardId":{"type":"string"},"deleted":{"type":"boolean"}}}',
  updated_at = datetime('now')
WHERE group_name = 'anki' AND id = 'delete_card';

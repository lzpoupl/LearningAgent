-- asset 工具契约：补齐 asset.list / asset.search / asset.read 的 parameters / returns。
-- 本文件只在 user_version 从 6 推进到 7 时执行一次，无需幂等。

UPDATE tool SET
  description = '列出可访问的非结构化学习资产，可按资料目录与类型过滤。',
  parameters = '{"type":"object","description":"列出可访问的非结构化学习资产，可按资料目录与类型过滤。","properties":{"bucket":{"type":"string","description":"只列出来自该资料目录的资产；省略时跨全部目录。"},"kind":{"type":"string","enum":["pdf","slides","note","image","document","other"],"description":"只返回指定类型的资产。"},"limit":{"type":"integer","description":"返回数量上限，缺省 50，最大 200。"},"offset":{"type":"integer","description":"跳过前若干条，用于翻页，缺省 0。"}}}',
  returns = '{"type":"object","properties":{"assets":{"type":"array","items":{"type":"object"}},"count":{"type":"integer"},"hasMore":{"type":"boolean"}}}',
  updated_at = datetime('now')
WHERE group_name = 'asset' AND id = 'list';

UPDATE tool SET
  description = '按关键字检索资产：匹配文件名，并搜索文本类资产的正文。',
  parameters = '{"type":"object","description":"按关键字检索资产：匹配文件名，并搜索文本类资产的正文。","properties":{"keyword":{"type":"string","description":"检索关键字，大小写不敏感。"},"bucket":{"type":"string","description":"限定检索范围的资料目录；省略时跨全部目录。"},"kind":{"type":"string","enum":["pdf","slides","note","image","document","other"],"description":"只检索指定类型的资产。"},"limit":{"type":"integer","description":"返回数量上限，缺省 50，最大 200。"},"offset":{"type":"integer","description":"跳过前若干条，用于翻页，缺省 0。"}},"required":["keyword"]}',
  returns = '{"type":"object","properties":{"matches":{"type":"array","items":{"type":"object"}},"count":{"type":"integer"},"hasMore":{"type":"boolean"}}}',
  updated_at = datetime('now')
WHERE group_name = 'asset' AND id = 'search';

UPDATE tool SET
  description = '按 id 读取文本类资产的正文，可分页。',
  parameters = '{"type":"object","description":"按 id 读取文本类资产的正文，可分页。","properties":{"assetId":{"type":"string","description":"资产 id，形如 /<bucket>/<path>。"},"offset":{"type":"integer","description":"按字符跳过的偏移量，缺省 0。"},"limit":{"type":"integer","description":"最多返回的字符数，缺省 20000，最大 100000。"}},"required":["assetId"]}',
  returns = '{"type":"object","properties":{"asset":{"type":"object"},"content":{"type":"string"},"offset":{"type":"integer"},"returnedChars":{"type":"integer"},"totalChars":{"type":"integer"},"truncated":{"type":"boolean"}}}',
  updated_at = datetime('now')
WHERE group_name = 'asset' AND id = 'read';

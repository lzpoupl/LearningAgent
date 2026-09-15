# 数据库迁移

## 文件约定

在 `src-tauri/migrations` 目录存放 `.sql` 文件，命名格式为 6 位数字，例如 `000001.sql`、
`000002.sql`。编号即迁移版本号，必须单调递增。

- 每个迁移文件只执行一次。
- **不允许修改已提交的迁移文件**；后续结构或数据的任何变更都新增更高编号的文件。
- 迁移按编号升序执行，后一个文件可以依赖前一个文件建立的表、列与数据。
- 因为只执行一次，迁移**不再要求幂等**，可以使用裸 `CREATE TABLE`、`ALTER TABLE`、
  普通 `INSERT` 等非幂等语句。

## 增量重放机制

用 SQLite 的 `PRAGMA user_version` 记录「已经应用到哪个编号」，每次启动时只执行编号更大的
文件，因此不再重放历史迁移。

启动流程：

1. 读取当前版本：`PRAGMA user_version`。全新数据库为 `0`。
2. build.rs 在编译期扫描 `migrations` 目录，按编号升序生成迁移清单：
   `MIGRATIONS: &[(u32, &str)]`，数字取文件名前 6 位，SQL 用 `include_str!` 引入。
3. 遍历清单，跳过 `number <= current` 的文件；对 `number > current` 的文件依次执行：
    - `BEGIN`
    - `conn.execute_batch(sql)`
    - `PRAGMA user_version = <number>`
    - `COMMIT`
4. 全部执行完毕后，`user_version` 等于迁移清单中的最大编号。

要点：

- 版本号推进与 SQL 执行在同一个事务内，迁移失败时整体回滚，`user_version` 保持不变，
  下次启动会重试同一文件，不会跳过。
- `PRAGMA user_version` 的取值只能是整数，不能参数化；写入前对文件名解析出的数字做校验，
  再拼接进语句。
- 新库从 `0` 开始依次执行全部文件；老库只执行新增的文件。

## build.rs

编译期遍历 `migrations`，按文件名（编号）升序生成 `migrations.rs`，内容形如：

```rust
pub const MIGRATIONS: &[(u32, &str)] = &[
    (1, include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/migrations/000001.sql"))),
    (2, include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/migrations/000002.sql"))),
];
```

- 文件名必须是 6 位数字加 `.sql`；无法解析出编号的文件应让构建直接失败，避免被静默忽略。
- 生成文件写入 `OUT_DIR`，由 `repository/db.rs` 通过 `include!` 引入。
- 对 `migrations` 目录声明 `cargo:rerun-if-changed`，保证新增迁移后触发重新生成。

## 与旧机制的差异

- 旧机制每次启动重放全部 SQL，因此要求每个文件幂等（`IF NOT EXISTS` / `INSERT OR IGNORE`），
  且不支持裸 `ALTER TABLE`。
- 新机制下这些限制全部取消：迁移只跑一次，可以直接 `ALTER TABLE ADD COLUMN`、
  直接 `INSERT`，无需可重复执行。
- 禁止修改已提交文件，变更以追加编号的方式表达，迁移历史由文件编号与 git 共同保留。

## 老旧数据库

早期若存在由旧机制建立的数据库，其 `user_version` 为 `0`，会被新机制视为全新数据库并
从头执行全部迁移。目前尚无持久化数据库（开发态使用内存库），无需为其做兼容处理；
将来若需要，可在迁移机制外单独做一次版本探测与回填。

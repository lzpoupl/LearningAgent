在src-tauri/migrations目录存放.sql文件，命名格式为:
6位数字.sql，例如：000001.sql
目的是做到每一次数据库结构变更的时间都能被记录

在build.rs中配置：编译期遍历migrations中所有sql，然后生成migrations.rs，通过`include_str!`宏将sql字符串引入代码

迁移在每次启动时全部重放，因此每个.sql文件都必须幂等：建表与建索引使用`IF NOT EXISTS`，播种使用`INSERT OR IGNORE`。SQLite不支持`ADD COLUMN IF NOT EXISTS`，新增列直接合并进对应表的`CREATE TABLE IF NOT EXISTS`定义，不使用裸`ALTER TABLE`。

在src-tauri/migrations目录存放.sql文件，命名格式为:
6位数字.sql，例如：000001.sql
目的是做到每一次数据库结构变更的时间都能被记录

在build.rs中配置：编译期遍历migrations中所有sql，然后生成migrations.rs，通过`include_str!`宏将sql字符串引入代码

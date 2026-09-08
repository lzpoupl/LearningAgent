use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    tauri_build::build();

    // 扫描 migrations 目录下的所有 .sql 文件，按文件名（时间戳）升序生成迁移清单，
    // 供 repository/db.rs 遍历执行。文件名格式：yyyymmddhhmmss.sql。
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR 未设置"));
    let migrations_dir = manifest_dir.join("migrations");

    println!("cargo:rerun-if-changed={}", migrations_dir.display());

    let mut files: Vec<String> = fs::read_dir(&migrations_dir)
        .expect("无法读取 migrations 目录")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sql"))
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(String::from)
        })
        .collect();

    files.sort();

    let mut generated = String::from("pub const MIGRATIONS: &[&str] = &[\n");
    for name in &files {
        generated.push_str(&format!(
            "    include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/migrations/{name}\")),\n"
        ));
    }
    generated.push_str("];\n");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR 未设置"));
    fs::write(out_dir.join("migrations.rs"), generated).expect("无法写入生成的 migrations.rs");
}

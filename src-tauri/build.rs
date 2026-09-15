use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    tauri_build::build();

    // 扫描 migrations 目录下的所有 .sql 文件，按文件名编号升序生成迁移清单，
    // 供 repository/db.rs 按版本号增量执行。文件名格式：6 位数字加 .sql。
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR 未设置"));
    let migrations_dir = manifest_dir.join("migrations");

    println!("cargo:rerun-if-changed={}", migrations_dir.display());

    let mut files: Vec<(u32, String)> = fs::read_dir(&migrations_dir)
        .expect("无法读取 migrations 目录")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sql"))
        .map(|path| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("迁移文件名不是合法 UTF-8")
                .to_string();
            let number = parse_number(&name);
            (number, name)
        })
        .collect();

    files.sort();

    for window in files.windows(2) {
        if window[0].0 == window[1].0 {
            panic!("迁移编号重复：{} 与 {}", window[0].1, window[1].1);
        }
    }

    let mut generated = String::from("pub const MIGRATIONS: &[(u32, &str)] = &[\n");
    for (number, name) in &files {
        generated.push_str(&format!(
            "    ({number}, include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/migrations/{name}\"))),\n"
        ));
    }
    generated.push_str("];\n");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR 未设置"));
    fs::write(out_dir.join("migrations.rs"), generated).expect("无法写入生成的 migrations.rs");
}

/// 从文件名解析 6 位数字编号，无法解析时让构建直接失败。
fn parse_number(name: &str) -> u32 {
    let stem = name
        .strip_suffix(".sql")
        .unwrap_or_else(|| panic!("迁移文件名必须以 .sql 结尾：{name}"));
    if stem.len() != 6 || !stem.bytes().all(|b| b.is_ascii_digit()) {
        panic!("迁移文件名必须是 6 位数字加 .sql，例如 000001.sql：{name}");
    }
    stem.parse().expect("迁移编号无法解析")
}

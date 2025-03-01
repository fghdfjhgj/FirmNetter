use cbindgen::Config;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // 告诉 Cargo 在哪里可以找到静态库
    println!("cargo:rustc-link-search=native=C:/Program Files/PostgreSQL/17/lib");
    println!("cargo:rustc-link-lib=static=pq");

    // 获取目标目录（根据构建模式，可能是 target/debug 或 target/release）
    let out_dir = if cfg!(debug_assertions) {
        PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()))
            .join("debug")
    } else {
        PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()))
            .join("release")
    };

    // 打印 OUT_DIR 以便了解生成文件的位置
    println!("The OUT_DIR is: {}", out_dir.display());

    // 获取当前 crate 的根目录
    let crate_dir =
        env::var("CARGO_MANIFEST_DIR").expect("Could not find Cargo manifest directory");

    // 加载配置文件，如果有的话
    let config = match Config::from_file("cbindgen.toml") {
        Ok(cfg) => cfg,
        Err(e) => panic!("Unable to load cbindgen.toml configuration: {:?}", e),
    };

    // 尝试生成绑定并写入到输出目录下的头文件中
    match cbindgen::generate_with_config(&crate_dir, config) {
        Ok(bindings) => {
            // 确保输出目录存在
            if let Some(parent) = out_dir.parent() {
                fs::create_dir_all(parent)
                    .expect("Unable to create parent directories for output file");
            }

            bindings.write_to_file(out_dir.join("FirmNetter.h"));
            println!("Successfully generated lib_tool.h in {}", out_dir.display());
        }
        Err(e) => {
            eprintln!("Failed to generate bindings: {:?}", e);
            std::process::exit(1);
        }
    }
}

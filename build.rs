use std::env;

use cbindgen::Config;
use std::path::PathBuf;
fn main() {
    // 告诉 Cargo 在哪里可以找到静态库
    println!("cargo:rustc-link-search=native=C:\\Program Files\\PostgreSQL\\17\\lib");

    // 指定要链接的静态库的名字（不带前缀 'lib' 和后缀）
    println!("cargo:rustc-link-lib=static=pq"); // 注意这里应该是 "pq" 而不是 "libpq"


    // 加载配置文件，如果有的话
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // 加载配置文件，如果有的话
    let config = Config::from_file("cbindgen.toml").expect("Unable to load cbindgen.toml configuration");

    // 生成绑定并写入到输出目录下的头文件中
    cbindgen::generate_with_config(crate_dir, config)
        .expect("Unable to generate bindings")
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("lib_tool.h"));
}
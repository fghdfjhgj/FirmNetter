fn main() {
    // 告诉 Cargo 在哪里可以找到静态库
    println!("cargo:rustc-link-search=native=C:\\Program Files\\PostgreSQL\\17\\lib");

    // 指定要链接的静态库的名字（不带前缀 'lib' 和后缀）
    println!("cargo:rustc-link-lib=static=pq"); // 注意这里应该是 "pq" 而不是 "libpq"
}
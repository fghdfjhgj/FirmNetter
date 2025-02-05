pub mod kernel {
    use std::ffi::c_char;
    use crate::other_utils::cstring_to_string;
    use crate::utils::utils::{exec, str_to_cstr};


    #[no_mangle]
    /// 解包图像文件的 C 兼容接口函数。
    ///
    /// 此函数用于调用外部 `magisk.exe` 工具来解包图像文件，并根据传入的标志参数决定是否执行特定操作或保留头部信息。
    ///
    /// # 参数
    ///
    ///* `filename` - 指向 C 字符串的指针，表示要解包的文件名。
    /// * `_n` - 布尔值，指示是否需要执行特定的解包操作。如果为 `true`，则传递 `-n` 参数给 `magisk.exe`。
    /// * `_h` - 布尔值，指示是否需要保留头部信息。如果为 `true`，则传递 `-h` 参数给 `magisk.exe`。
    ///
    /// # 返回值
    ///
    /// * `*mut c_char` - 返回输出信息。
    ///

    pub extern "C" fn unpack_img(file_name: *const c_char, _n: bool, _h: bool) -> *mut c_char {
        // 根据 _n 标志决定是否添加 "-n" 参数
        let a = if _n { "-n" } else { "" };
        // 根据 _h 标志决定是否添加 "-h" 参数
        let b = if _h { "-h" } else { "" };
        // 构建并执行 magisk.exe unpack 命令，返回命令执行的成功状态
        let a=exec(str_to_cstr(format!("magiskboot unpack {} {} {}", a, b ,cstring_to_string(file_name).unwrap())));
        match a.success {
           true => {
               // 如果命令执行成功，则返回 "OK"
               a.stderr
           },
           false => {
               // 如果命令执行失败，则返回 "FAIL"
               a.stderr
           }
        }
    }
    #[no_mangle]
    /// 将镜像重新打包
    ///
    /// 此函数通过调用外部的 magisk.exe 程序来重新打包图像。它允许用户指定是否需要添加特定的参数，
    /// 以及原始引导文件和输出文件的名称。
    ///
    /// 参数:
    /// - `_n`: 一个布尔值，决定是否添加 "-n" 参数到 magisk.exe pack 命令中。
    /// - `out_file_name`: 输出文件的名称，作为 C 风格字符串传递。
    /// - `origboot`: 原始引导文件的名称，作为 C 风格字符串传递。
    ///
    /// 返回:
    /// - 返回一个指向 C 风格字符串的指针，表示命令执行的成功状态。
    pub extern "C" fn repack_img(_n: bool, out_file_name: *const c_char, origboot: *const c_char) -> *mut std::os::raw::c_char {
        // 根据 _n 标志决定是否添加 "-n" 参数
        let a = if _n { "-n" } else { "" };
        // 构建并执行 magisk.exe pack 命令，返回命令执行的成功状态
        let a=exec(str_to_cstr(format!("magiskboot repack {} {} {}", a, cstring_to_string(origboot).expect("error"), cstring_to_string(out_file_name).expect("error"))));
        match a.success {
           true => {
               // 如果命令执行成功，则返回 "OK"
               a.stdout
           },
           false => {
               // 如果命令执行失败，则返回 "FAIL"
               a.stderr
           }
        }
    }
    /// 验证文件完整性
    ///
    /// 此函数通过调用外部的 `magisk.exe` 工具来验证文件的完整性它使用 C ABI 来允许从 C 代码中调用，
    /// 主要用于与 C 语言环境或其他限制性环境交互
    ///
    /// # 参数
    ///
    /// * `file` - 指向文件路径的 C 风格字符串指针需要验证的文件路径
    /// * `pom` - 指向另一个文件路径的 C 风格字符串指针，通常用于指定验证所需的额外参数或配置文件
    ///
    /// # 返回值
    ///
    /// 返回一个指向 C 风格字符串的指针，该字符串包含验证过程的标准输出结果
    /// 如果在转换字符串或执行过程中遇到错误，此函数将返回一个错误信息
    ///
    /// # 安全性
    ///
    /// 调用此函数时需要确保传入的字符串指针有效，且指向的字符串在函数调用过程中保持有效
    /// 由于此函数直接构造命令行命令并执行，应确保输入参数不会导致命令行注入安全风险
    #[no_mangle]
    pub extern "C" fn verify(file: *const c_char, pom: *const c_char) -> *const c_char {
        // 构造并执行验证命令，返回验证结果的标准输出
        let a=exec(str_to_cstr(format!("magiskboot verify {} {} ", cstring_to_string(file).expect("error"), cstring_to_string(pom).expect("error"))));
        match a.success {
           true => {
               // 如果命令执行成功，则返回 "OK"
               a.stdout
           },
           false => {
               // 如果命令执行失败，则返回 "FAIL"
               a.stderr
           }
        }

    }
    /// 对图像文件进行签名
    ///
    /// 该函数通过调用外部的 `magiskboot` 工具对指定的图像文件进行签名
    /// 使用 C 型链接规范，防止符号名 mangling，以便在其他语言中调用
    ///
    /// # 参数
    ///
    /// * `file`: *const c_char - 图像文件的路径
    /// * `name`: *const c_char - 签名的名称
    /// * `pem`: *const c_char - PEM 文件路径，包含签名密钥
    ///
    /// # 返回
    ///
    /// * `*const c_char` - 签名操作的标准输出
    ///
    /// # 安全
    ///
    /// 调用此函数时需要确保传入的指针有效且可读，否则可能导致未定义行为
    #[no_mangle]
    pub extern "C" fn sign_img(file: *const c_char, name: *const c_char, pem: *const c_char) -> *const c_char {
        // 执行签名命令并返回其标准输出
        // 使用 `format!` 构建命令字符串，通过 `str_to_cstr` 转换为 C 型字符串
        // `cstring_to_string` 用于将 C 型字符串转换为 Rust 字符串
        // `expect` 处理转换时可能发生的错误
        let a=exec(str_to_cstr(format!("magiskboot sign {} {} {}", cstring_to_string(file).expect("error"), cstring_to_string(name).expect("error"), cstring_to_string(pem).expect("error"))));
        match a.success {
           true => {
               // 如果命令执行成功，则返回 "OK"
               a.stdout
           },
           false => {
               // 如果命令执行失败，则返回 "FAIL"
               a.stderr
           }
        }
    }
    #[no_mangle]
    pub extern "C" fn extract(payload_bin: *const c_char, partition: *const c_char,  outfile:*const c_char)->*const c_char{
        let a=exec(str_to_cstr(format!("magiskboot extract {} {} {}", cstring_to_string(payload_bin).expect("error"), cstring_to_string(partition).expect("error"), cstring_to_string(outfile).expect("error"))));
        match a.success {
           true => {
               // 如果命令执行成功，则返回 "OK"
               a.stdout
           },
           false => {
               // 如果命令执行失败，则返回 "FAIL"
               a.stderr
           }
        }
    }

}


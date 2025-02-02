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
        exec(str_to_cstr(format!("magisk.exe unpack {} {} {}", a, b ,cstring_to_string(file_name).unwrap()))).stdout
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
        exec(str_to_cstr(format!("magisk.exe pack {} {} {}", a, cstring_to_string(origboot).expect("error"), cstring_to_string(out_file_name).expect("error")))).stdout
    }


}


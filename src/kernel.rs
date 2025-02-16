pub mod kernel {
    use crate::other_utils::cstring_to_string;
    use crate::utils::utils::exec;
    use std::ffi::c_char;


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
        let a=exec(format!("magiskboot unpack {} {} {}", a, b ,cstring_to_string(file_name)));
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
        let a=exec(format!("magiskboot repack {} {} {}", a, cstring_to_string(origboot), cstring_to_string(out_file_name)));
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
        let a=exec(format!("magiskboot verify {} {} ", cstring_to_string(file), cstring_to_string(pom)));
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
        let a=exec(format!("magiskboot sign {} {} {}", cstring_to_string(file), cstring_to_string(name), cstring_to_string(pem)));
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
   /// 使用no_mangle属性以防止名称修饰，确保函数名在外部保持不变
    /// 使用extern "C"来指定函数的调用约定与C语言兼容
    /// 这个函数接收三个C风格的字符串指针作为参数，并返回一个C风格的字符串指针
    /// 参数payload_bin指向一个表示payload二进制文件路径的C风格字符串
    /// 参数partition指向一个表示分区信息的C风格字符串
    /// 参数outfile指向一个表示输出文件路径的C风格字符串
    /// 函数的作用是调用magiskboot工具来提取payload中的特定分区，并将结果保存到输出文件中

    #[no_mangle]
    pub extern "C" fn extract(payload_bin: *const c_char, partition: *const c_char,  outfile:*const c_char)->*const c_char{
        // 构造命令行字符串并执行magiskboot extract命令
        let a=exec(format!("magiskboot extract {} {} {}", cstring_to_string(payload_bin), cstring_to_string(partition), cstring_to_string(outfile)));
        // 根据命令执行结果返回相应的字符串
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
    /// 使用no_mangle属性防止符号名称被修改，确保外部调用的一致性
    /// 使用extern "C"指定函数的调用约定与C语言相同，以便其他语言可以调用此函数
    #[no_mangle]
    pub extern "C" fn hexpatch(file: *const c_char, hexpattern1: *const c_char, hexpattern2: *const c_char) -> *const c_char {
        // 将文件路径和十六进制模式从C字符串转换为Rust字符串，并执行hexpatch命令
        let a = exec(format!("magiskboot hexpatch {} {} {}", cstring_to_string(file), cstring_to_string(hexpattern1), cstring_to_string(hexpattern2)));

        // 根据命令执行结果返回相应的输出
        match a.success {
            true => {
                // 如果命令执行成功，则返回 "OK"
                a.stdout
            },
            false => {
                // 如果命令执行失败，则返回错误信息
                a.stderr
            }
        }
    }
    // 使用no_mangle属性防止符号名称被修改，确保外部C代码可以调用此函数
    // 使用extern "C"指定函数使用C语言的调用约定
    /// 增加或修改内核命令行参数
    ///
    /// # 参数
    ///
    /// * `file` - 指向一个以null结尾的C字符串，表示目标文件路径
    /// * `commands` - 指向一个以null结尾的C字符串，表示要增加或修改的命令行参数
    /// * `"patch" `-表示修补boot(命令行参数的示例)
    /// 这里所有参数都必须带""(引号)
    /// # 返回
    ///

    #[no_mangle]
    pub extern "C" fn incpio(file: *const c_char, commands: *const c_char) -> *const c_char {

        // 构造并执行命令，处理可能的错误
        let a = exec(format!("magiskboot incpio {} {}", cstring_to_string(file), cstring_to_string(commands)));

        // 根据命令执行结果返回相应的值
        match a.success {
            true => {
                // 如果命令执行成功，则返回 "OK"
                a.stdout
            },
            false => {
                // 如果命令执行失败，则返回错误信息
                a.stderr
            }
        }
    }
    /// 使用no_mangle属性以防止名称修饰，确保函数符号在编译后保持原样
    /// 使用extern "C" ABI标记，使函数能够被C语言代码调用
    /// 函数dtb用于处理设备树blob（DTB）文件的操作
    #[no_mangle]
    pub extern "C" fn dtb (file: *const c_char, action:*const c_char, args: *const c_char)->*const c_char{
        // 将C字符串参数转换为Rust字符串，并构造magiskboot dtb命令
        let a=exec(format!("magiskboot dtb {} {} {}", cstring_to_string(file), cstring_to_string(action), cstring_to_string(args)));
        // 根据命令执行结果返回相应的C字符串
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
    // 导出一个C接口，用于根据条件分割文件
    #[no_mangle]
    pub extern "C" fn split(_n:bool, file:*const c_char)->*const c_char{
        // 根据_n的值构造命令参数，-n表示启用特定模式
        let b = if _n { "-n" } else { "" };
        // 构造并执行magiskboot split命令
        let a=exec(format!("magiskboot split {} {} ",b, cstring_to_string(file)));
        // 根据命令执行结果返回相应的C字符串
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
    /// 执行hsa1命令的函数
    ///
    /// 此函数通过调用外部的`magiskboot`工具来执行`hsa1`命令，该命令的具体逻辑未在代码中展示。
    /// 主要负责将文件路径从C字符串转换为Rust字符串，执行命令，然后根据命令执行结果返回相应的C字符串。
    ///
    /// # 参数
    /// * `file`: *const c_char - 指向文件路径的C字符串指针
    ///
    /// # 返回值
    /// *const c_char - 指向命令执行结果的C字符串指针，成功时为"OK"，失败时为"FAIL"
    #[no_mangle]
    pub extern "C" fn hsa1(file: *const c_char)->*const c_char{
        // 执行命令并获取结果
        let a=exec(format!("magiskboot hsa1 {} ", cstring_to_string(file)));
        // 根据命令执行结果返回相应的C字符串
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
    /// 导出一个名为 magisk_clean 的 C 接口函数，用于执行 Magisk 清理操作
    #[no_mangle]
    pub extern "C" fn magisk_clean() -> *const c_char {
        // 执行 "magiskboot cleanup" 命令，并将结果转换为 C 语言字符串
        let a = exec("magiskboot cleanup");

        // 根据命令执行结果决定返回值
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
    /// 使用no_mangle属性以防止名称修饰，确保函数名在外部保持不变
    /// 使用extern "C"来指定函数的调用约定与C语言兼容
    /// 这个函数用于解压缩文件，接受输入文件和输出文件的路径作为参数
    /// 返回一个指向C类型字符串的指针，表示操作结果
    #[no_mangle]
    pub extern "C" fn decompress(infile: *const c_char, outfile: *const c_char)->*const c_char{
        // 构造并执行解压缩命令
        let a=exec(format!("magiskboot decompress {} {} ", cstring_to_string(infile), cstring_to_string(outfile)));
        // 根据命令执行结果返回相应的字符串
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
    /// 使用no_mangle属性以防止名称修饰，确保函数名在外部保持不变
    /// 使用extern "C"来指定函数的调用约定与C语言兼容
    /// 这个函数用于压缩文件，接受三个参数：压缩类型、输入文件和输出文件
    /// 返回一个指向C风格字符串的指针，表示操作结果
    #[no_mangle]
    pub extern "C" fn compress(zip: *const c_char, infile: *const c_char, outfile: *const c_char) -> *const c_char {
        // 构造压缩命令并执行
        let a = exec(format!("magiskboot compress={} {} {} ", cstring_to_string(zip), cstring_to_string(infile), cstring_to_string(outfile)));

        // 根据命令执行结果返回相应的字符串
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


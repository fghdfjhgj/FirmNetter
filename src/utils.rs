pub mod utils {
    use std::ffi::{CStr, CString, c_char};
    use std::process::{Command, Stdio};
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{fs, ptr};
    /// 定义一个对外的 C 接口，执行外部命令
    /// 该接口使用原始指针和长度来传递命令字符串，以适应 C 语言的调用习惯
    #[repr(C)]
    pub struct CommandResult {
        pub success: bool,
        pub stdout: *mut c_char,
        pub stderr: *mut c_char,
    }

    impl CommandResult {
        fn new(success: bool, stdout: *mut c_char, stderr: *mut c_char) -> Self {
            CommandResult { success, stdout, stderr }
        }

        // 提供一个方法来安全地释放由 CommandResult 包含的 C 字符串
        pub fn free(&self) {
            unsafe {
                if !self.stdout.is_null() {
                    let _ = CString::from_raw(self.stdout);
                }
                if !self.stderr.is_null() {
                    let _ = CString::from_raw(self.stderr);
                }
            }
        }
    }

    /// 将 C 风格字符串转换为 Rust `String`。
    ///
    /// # 参数
    ///
    /// * `s` - 指向 C 风格字符串的指针 (`*const c_char`)。该字符串应以空字符结尾。
    ///
    /// # 返回值
    ///
    /// * `Ok(String)` - 如果转换成功，返回包含转换后字符串的 `Result::Ok`。
    /// * `Err(std::str::Utf8Error)` - 如果输入的 C 字符串包含无效的 UTF-8 序列，则返回 `Result::Err` 包含一个 `std::str::Utf8Error`。
    ///
    /// # 安全性
    ///
    /// 该函数使用了 `unsafe` 块来进行裸指针操作。调用者必须确保传入的指针是有效的，并且指向一个以空字符结尾的 C 风格字符串。如果指针为空，函数将安全地返回一个空字符串。
    #[no_mangle]
    pub fn cstring_to_string(s: *const c_char) -> Result<String, std::str::Utf8Error> {
        unsafe {
            if s.is_null() {
                return Ok(String::new());
            }
            let c_str = CStr::from_ptr(s);
            c_str.to_str().map(|s| s.to_owned())
        }
    }

    /// 释放 `CommandResult` 结构体中包含的 C 字符串内存
    #[no_mangle]
    pub extern "C" fn free_command_result(result: CommandResult) {
        result.free();
    }

    /// 执行外部命令并返回结果
    ///
    /// # 参数
    ///
    /// * `command` - 指向 C 风格字符串的指针 (`*const c_char`)，表示要执行的命令。
    ///
    /// # 返回值
    ///
    /// 返回一个 `CommandResult` 结构体，包含命令执行的结果。
    #[no_mangle]
    pub extern "C" fn exec(command: *const c_char) -> CommandResult {
        // 安全检查：确保传入的指针是有效的。
        if command.is_null() {
            return CommandResult::new(false, ptr::null_mut(), ptr::null_mut());
        }

        // 将 *const c_char 转换为 String
        let com = match unsafe { CStr::from_ptr(command).to_string_lossy().into_owned() } {
            s if !s.is_empty() => s,
            _ => return CommandResult::new(false, ptr::null_mut(), ptr::null_mut()),
        };
        #[cfg(target_os = "windows")]
        let shell_command = "powershell.exe";
        #[cfg(not(target_os = "windows"))]
        let shell_command = "sh"; // 注意这里改为 'sh' 而不是 'bin/bash'

        #[cfg(target_os = "windows")]
        let arg_prefix = "-NoProfile -ExecutionPolicy Bypass -Command";
        #[cfg(not(target_os = "windows"))]
        let arg_prefix = "-c";
        // 构造完整的命令字符串
        #[cfg(target_os = "windows")]
        let full_command = format!("\"{}\"", com); // 使用双引号包裹命令以确保正确解析
        #[cfg(not(target_os = "windows"))]
        let full_command = com; // 对于非Windows系统，直接使用原始命令

        // 构造完整的命令字符串，首先设置代码页为 65001 (UTF-8)，然后执行用户提供的命令
        // 执行命令并获取输出和错误信息
        let output = match Command::new(shell_command)
            .arg(arg_prefix) // 传递参数前缀
            .arg(&full_command) // 传递命令字符串
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(output) => output,
            Err(_) => return CommandResult::new(false, ptr::null_mut(), ptr::null_mut()),
        };

        // 将标准输出转换为 C 兼容的字符串
        let stdout_cstring = CString::new(String::from_utf8_lossy(&output.stdout).into_owned()).unwrap_or_else(|_| CString::new("").unwrap());
        let stdout_ptr = stdout_cstring.into_raw();

        // 将标准错误转换为 C 兼容的字符串
        let stderr_cstring = CString::new(String::from_utf8_lossy(&output.stderr).into_owned()).unwrap_or_else(|_| CString::new("").unwrap());
        let stderr_ptr = stderr_cstring.into_raw();

        CommandResult::new(output.status.success(), stdout_ptr, stderr_ptr)
    }

    /// 释放 `CString` 内存的函数
    ///
    /// 这个函数是为了提供给 C 语言代码使用的，因此使用 `extern "C"` 声明。
    ///
    /// # 参数
    ///
    /// * `ptr` - 一个指向 C 字符串的指针。
    #[no_mangle]
    pub extern "C" fn free_cstring(ptr: *mut c_char) {
        // 使用 `unsafe` 块，因为涉及到直接操作原始指针
        unsafe {
            // 检查指针是否为空，避免传入无效指针导致的错误
            if ptr.is_null() {
                return;
            }
            // 通过 `from_raw` 方法将指针转换回 `CString`，这会自动释放内存
            // 这里使用 `_` 来忽略掉 `CString` 实例，因为我们只关心内存释放
            let _ = CString::from_raw(ptr);
        }
    }

    /// 释放并重置 C 字符串指针
    ///
    /// 该函数旨在与 C 代码互操作，通过释放动态分配的 C 字符串并将其指针设置为 `NULL` 来避免内存泄漏。
    /// 它使用 `CString::from_raw` 从原始指针获取所有权并安全地释放内存，然后重置指针。
    ///
    /// # 参数
    ///
    /// * `ptr` - 一个指向 C 字符串的指针引用，该字符串将被释放并重置。
    ///
    /// # 安全性
    ///
    /// 此函数涉及不安全代码块，因为它处理原始指针。必须确保在释放内存后指针不会再次被使用，以避免悬挂指针。
    /// 通过将指针设置为 `NULL`，我们确保了这一点。
    #[no_mangle]
    pub extern "C" fn free_and_reset_c_string(ptr: &mut *const c_char) {
        unsafe {
            if !ptr.is_null() {
                // 从原始指针获取所有权并释放内存
                let _ = CString::from_raw(*ptr as *mut _);
                // 重置指针为 `NULL`，避免悬挂指针
                *ptr = ptr::null();
            }
        }
    }

    /// 将 Rust 字符串转换为 C 风格的字符串
    ///
    /// 此函数接收一个 Rust `String` 类型的参数，并将其转换为 `*const c_char` 类型，
    /// 即 C 语言中字符串的指针类型。这一转换是为了在 Rust 代码中调用 C 语言库函数时，
    /// 能够传递字符串参数给 C 函数。
    ///
    /// # 参数
    ///
    /// * `s` - 一个 `String` 类型的参数，代表需要转换的 Rust 字符串。
    ///
    /// # 返回值
    ///
    /// 返回一个 `*const c_char` 类型的指针，指向转换后的 C 风格字符串。
    ///
    /// # 安全性
    ///
    /// 调用此函数的代码需要确保在使用完指针后正确地释放内存，以避免内存泄漏。
    /// 此外，由于返回的是一个原始指针，使用时需要确保不会造成未定义行为，例如
    /// 解引用悬挂指针等。
    #[no_mangle]
    pub  fn str_to_cstr(s: String) -> *mut c_char {
        // 使用 `CString::new` 创建一个新的 C 风格字符串，并自动处理转换过程中的错误。
        let a = CString::new(s).unwrap();
        // 通过 `into_raw` 方法获取原始指针，注意此时所有权转移给了调用者。
        a.into_raw()
    }

    /// 返回当前时间自 UNIX_EPOCH（1970年1月1日00:00:00 UTC）以来的天数
    ///
    /// # 返回值
    ///
    /// 返回一个 `i32` 类型的值，表示当前时间自 UNIX_EPOCH 以来的天数。
    /// 如果出现错误（例如，当前时间在 UNIX_EPOCH 之前），则返回 1 表示错误或无效的日期。
    #[no_mangle]
    pub extern "C" fn Get_time() -> i32 {
        // 获取当前系统时间
        let current_time = SystemTime::now();

        // 尝试计算当前时间与 UNIX_EPOCH 的时间差
        match current_time.duration_since(UNIX_EPOCH) {
            // 如果时间差计算成功
            Ok(duration) => {
                // 将时间差转换为秒数
                let seconds = duration.as_secs();

                // 将秒数转换为天数
                let days = seconds / 60 / 60 / 24;
                // 将天数作为 `i32` 类型返回
                days as i32
            }
            // 如果出现错误（例如，当前时间在 UNIX_EPOCH 之前）
            Err(_e) => {
                // 返回 1 表示错误或无效的日期
                1
            }
        }
    }
    /// 检查指定路径的文件是否存在
    ///
    /// # Parameters
    ///
    /// * `file_path` - 文件路径的C字符串指针
    ///
    /// # Returns
    ///
    /// * `1` - 文件存在
    /// * `0` - 文件不存在
    /// * `-1` - 发生其他错误
    #[no_mangle]
    pub extern "C" fn check_file(file_path: *const c_char) -> i32 {
        // 将C字符串转换为Rust字符串
        let file_path_str = cstring_to_string(file_path).unwrap();
        match fs::metadata(file_path_str) {
            Ok(_) => 1, // 文件存在，返回1
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    0 // 文件不存在，返回0
                } else {
                    -1 // 其他错误发生，返回-1
                }
            }
        }
    }
}

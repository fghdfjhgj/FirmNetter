pub mod utils {
    use encoding_rs::GBK;
    use std::ffi::{CStr, CString, c_char};
    use std::path::Path;
    use std::process::{Command, Output, Stdio};
    use std::{fs, ptr};
    #[repr(C)]
    pub struct CCommandResult {
        pub success: bool,
        pub c_stdout: *mut c_char,
        pub c_stderr: *mut c_char,
    }
    impl CCommandResult {
        fn new(success: bool, c_stdout: *mut c_char, c_stderr: *mut c_char) -> Self {
            CCommandResult {
                success,
                c_stdout,
                c_stderr,
            }
        }

        // 提供一个方法来安全地释放由 CommandResult 包含的 C 字符串

        pub fn free(&self) {
            unsafe {
                if !self.c_stdout.is_null() {
                    let _ = CString::from_raw(self.c_stdout);
                }
                if !self.c_stderr.is_null() {
                    let _ = CString::from_raw(self.c_stderr);
                }
            }
        }
    }

    /// 定义一个对外的 C 接口，执行外部命令
    /// 该接口使用原始指针和长度来传递命令字符串，以适应 C 语言的调用习惯

    pub struct CommandResult {
        pub success: bool,
        pub stdout: String,
        pub stderr: String,
    }
    impl CommandResult {
        fn new(success: bool, stdout: String, stderr: String) -> Self {
            CommandResult {
                success,
                stdout,
                stderr,
            }
        }
        pub fn clear(&mut self) {
            self.success = false;
            self.stdout.clear(); // 使用 String 的 clear 方法来清空字符串
            self.stderr.clear(); // 使用 String 的 clear 方法来清空字符串
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

    pub fn cstring_to_string(s: *const c_char) -> String {
        unsafe {
            if s.is_null() {
                return String::new();
            }
            let c_str = CStr::from_ptr(s);
            // 使用 to_string_lossy 确保总是返回一个有效的 String
            c_str.to_string_lossy().into_owned()
        }
    }

    /// 释放 `CCommandResult` 结构体中包含的 C 字符串内存
    #[unsafe(no_mangle)]
    pub extern "C" fn free_command_result(result: CCommandResult) {
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
    pub fn exec<T: AsRef<str>>(command: T) -> CommandResult {
        let com_str = command.as_ref();

        // 根据操作系统选择 shell 和参数
        #[cfg(target_os = "windows")]
        let (shell, arg) = ("cmd", "/C");
        #[cfg(not(target_os = "windows"))]
        let (shell, arg) = ("sh", "-c");

        // 执行命令并捕获输出
        let output: Output = Command::new(shell)
            .arg(arg)
            .arg(com_str)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();

        // 根据操作系统处理编码
        #[cfg(target_os = "windows")]
        let stdout = decode_output(&output.stdout);
        #[cfg(not(target_os = "windows"))]
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

        #[cfg(target_os = "windows")]
        let stderr = decode_output(&output.stderr);
        #[cfg(not(target_os = "windows"))]
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        CommandResult::new(output.status.success(), stdout, stderr)
    }

    /// 在 Windows 下使用 GBK 解码字节流
    #[cfg(target_os = "windows")]
    fn decode_output(output: &[u8]) -> String {
        use encoding::all::GBK;
        use encoding::{DecoderTrap, Encoding};

        match GBK.decode(output, DecoderTrap::Strict) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Decoding issues encountered, falling back to UTF-8.");
                String::from_utf8_lossy(output).into_owned()
            }
        }
    }
    // 外部 C 接口
    #[unsafe(no_mangle)]
    pub extern "C" fn c_exec(command: *const c_char) -> CCommandResult {
        let command_str = cstring_to_string(command);
        let result = exec(command_str);
        CCommandResult::new(
            result.success,
            CString::new(result.stdout).unwrap().into_raw(),
            CString::new(result.stderr).unwrap().into_raw(),
        )
    }

    /// 释放 `CString` 内存的函数
    ///
    /// 这个函数是为了提供给 C 语言代码使用的，因此使用 `extern "C"` 声明。
    ///
    /// # 参数
    ///
    /// * `ptr` - 一个指向 C 字符串的指针。
    #[unsafe(no_mangle)]
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
    #[unsafe(no_mangle)]
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
    pub fn str_to_cstr<T: AsRef<str>>(s: T) -> *mut c_char {
        // 尝试将输入转换为 CString
        match CString::new(s.as_ref()) {
            Ok(c_string) => c_string.into_raw(), // 转换成功，返回原始指针
            Err(_) => ptr::null_mut(),           // 如果包含 NUL 字符，返回空指针
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
    #[unsafe(no_mangle)]
    pub extern "C" fn c_check_file(file_path: *const c_char) -> i32 {
        // 将 C 字符串转换为 Rust 字符串
        let file_path_str = unsafe {
            match CStr::from_ptr(file_path).to_str() {
                Ok(s) => s,
                Err(_) => return -1, // 如果转换失败，返回-1
            }
        };

        check_file(file_path_str)
    }

    /// 检查文件是否存在。
    ///
    /// 该函数是一个泛型函数，支持多种字符串类型作为输入参数。它会尝试获取指定路径的文件元数据，
    /// 并根据结果返回相应的整数值。
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径，可以是任何实现了 `AsRef<Path>` 特性的类型。
    ///
    /// # 返回值
    ///
    /// * `1` - 文件存在。
    /// * `0` - 文件不存在。
    /// * `-1` - 其他错误发生（例如权限问题等）。
    ///
    pub fn check_file<T: AsRef<Path>>(file_path: T) -> i32 {
        match fs::metadata(file_path) {
            Ok(_) => 1,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    0
                } else {
                    -1
                }
            }
        }
    }

    /// 将字符串按行分割成向量
    ///
    /// 该函数接受一个字符串切片作为输入，并将其按照行分隔符分割成一个字符串向量
    /// 主要用于处理需要按行显示或处理的文本数据
    ///
    /// # 参数
    ///
    /// * `text`: &str - 需要被分割的字符串切片
    ///
    /// # 返回值
    ///
    /// 返回一个`Vec<String>`，其中每个元素都是原字符串中的一行
    ///

    pub fn split_by_newline<T: AsRef<str>>(text: T) -> Vec<String> {
        // 使用as_ref()将T转换为&str，然后调用lines方法
        text.as_ref().lines().map(String::from).collect()
    }

    /// 将 UTF-8 编码的 C 字符串转换为 GBK 编码的 C 字符串。
    ///
    /// # 参数
    ///
    /// * `utf8_str` - 指向 UTF-8 编码的 C 字符串的指针 (`*const c_char`)。
    ///
    /// # 返回值
    ///
    /// * 成功时返回指向 GBK 编码的 C 字符串的指针 (`*mut c_char`)。
    /// * 如果编码转换过程中遇到错误或无法创建有效的 C 字符串，则返回空指针 (`ptr::null_mut()`)。
    ///
    /// # 注意事项
    ///
    /// * 该函数使用 `#[unsafe(no_mangle)]` 属性，确保其符号名称不会被 Rust 的名称修饰机制修改，以便与 C 语言代码互操作。
    /// * 在编码转换过程中，如果遇到无法转换的字符，会打印警告信息。
    #[unsafe(no_mangle)]
    pub extern "C" fn c_utf_8_str_to_gbk_str(utf8_str: *const c_char) -> *mut c_char {
        // 将 C 字符串转换为 Rust 字符串
        let input_str = unsafe { CStr::from_ptr(utf8_str).to_string_lossy().into_owned() };

        // 进行编码转换
        let (encoded_bytes, _, had_errors) = GBK.encode(&input_str);

        if had_errors {
            println!("Warning: encountered errors during encoding.");
        }

        // 将 GBK 编码的字节数组转换为 C 字符串
        match CString::new(encoded_bytes.into_owned()) {
            Ok(c_string) => c_string.into_raw(), // 返回 C 字符串指针
            Err(_) => ptr::null_mut(),           // 如果转换失败，返回空指针
        }
    }
    /// 将 UTF-8 编码的字符串转换为 GBK 编码的字符串。
    ///
    /// # 参数
    /// - `input`: 实现了 `AsRef<str>` trait 的类型，表示输入的 UTF-8 编码字符串。
    ///
    /// # 返回值
    /// 返回一个 `String` 类型，表示转换后的 GBK 编码字符串。
    ///
    /// # 注意事项
    /// - 如果输入字符串包含无法用 GBK 编码表示的字符，可能会出现乱码或错误。
    /// - 在编码过程中如果遇到错误，会输出警告信息。
    ///
    ///
    pub fn utf_8_str_to_gbk_str<T: AsRef<str>>(input: T) -> String {
        // 获取输入的引用字符串
        let utf8_str = input.as_ref();

        // 进行编码转换
        let (encoded_bytes, _, had_errors) = GBK.encode(utf8_str);

        if had_errors {
            println!("Warning: encountered errors during encoding.");
        }

        // 将编码后的字节序列转换为 String
        // 注意：这里直接将 GBK 编码的字节序列视为 UTF-8 可能会导致乱码或错误，
        // 但在某些情况下（如仅用于显示或特定用途），可以这样做。
        // 更好的做法是根据实际需求决定如何处理这些字节。

        // 直接将 GBK 编码的字节序列转换为 String
        // 这里我们使用 lossy 转换，以确保即使遇到无效的 UTF-8 字节也能生成一个 String
        String::from_utf8_lossy(&encoded_bytes).into_owned()
    }
}

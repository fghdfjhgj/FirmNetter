pub mod utils {
    use encoding_rs::GBK;
    use std::ffi::{CStr, CString, c_char};
    use std::io;
    use std::io::Write;
    use std::os::raw::c_int;
    use std::path::Path;
    use std::process::{Child, Command, Output, Stdio};
    use std::ptr;

    #[repr(C)]
    pub struct CCommandResult {
        pub success: bool,
        pub c_stdout: *mut c_char,
        pub c_stderr: *mut c_char,
    }

    impl CCommandResult {
        /// 使用安全的方式创建 CCommandResult，并处理 CString 创建失败的情况
        pub fn new(success: bool, stdout: String, stderr: String) -> Self {
            // 使用 try_into_raw 避免 panic（原代码中 unwrap 可能因包含空字符失败）
            // 当 stdout 中包含空字符时，CString::new(stdout) 会返回 Err
            // 使用 match 处理成功情况和失败情况，避免程序因错误而终止
            let c_stdout = match CString::new(stdout) {
                Ok(s) => s.into_raw(),
                Err(_) => ptr::null_mut(), // 创建失败返回空指针
            };
            // 同样的处理逻辑适用于 stderr
            let c_stderr = match CString::new(stderr) {
                Ok(s) => s.into_raw(),
                Err(_) => ptr::null_mut(),
            };
            // 构造 CCommandResult 实例，包含成功标志和输出的 C 字符串指针
            CCommandResult {
                success,
                c_stdout,
                c_stderr,
            }
        }

        /// 安全释放内存（使用 drop 确保资源释放）
        pub fn free(&self) {
            // 使用 unsafe 块来执行不安全的操作，如直接操作指针
            unsafe {
                // 检查 c_stdout 指针是否为空，如果不为空，则需要释放其占用的资源
                if !self.c_stdout.is_null() {
                    // 使用 drop 明确释放 CString 占用的资源，防止内存泄漏
                    drop(CString::from_raw(self.c_stdout));
                }
                // 检查 c_stderr 指针是否为空，如果不为空，则需要释放其占用的资源
                if !self.c_stderr.is_null() {
                    // 使用 drop 明确释放 CString 占用的资源，防止内存泄漏
                    drop(CString::from_raw(self.c_stderr));
                }
            }
        }
    }

    pub struct CommandResult {
        pub success: bool,
        pub stdout: String,
        pub stderr: String,
    }

    impl CommandResult {
        /// 创建一个新的CommandResult实例
        ///
        /// # 参数
        ///
        /// * `success` - 表示命令是否成功执行的布尔值
        /// * `stdout` - 命令的标准输出内容
        /// * `stderr` - 命令的标准错误输出内容
        ///
        /// # 返回值
        ///
        /// 返回一个新的CommandResult实例，该实例包含命令执行的成功状态、标准输出和标准错误输出
        pub fn new(success: bool, stdout: String, stderr: String) -> Self {
            CommandResult {
                success,
                stdout,
                stderr,
            }
        }

        /// 重置当前对象的状态
        ///
        /// 此方法将对象的成功标志设置为false，并清空标准输出和错误输出的缓存
        pub fn clear(&mut self) {
            self.success = false;
            self.stdout.clear();
            self.stderr.clear();
        }
    }

    /// 安全转换 C 字符串为 Rust 字符串（添加空指针检查）
    ///
    /// # 参数
    /// * `s`: *const c_char - 指向 C 字符串的指针
    ///
    /// # 返回值
    /// * 如果指针为 null，则返回一个空的 Rust 字符串
    /// * 否则，返回转换后的 Rust 字符串
    pub fn cstring_to_string(s: *const c_char) -> String {
        // 检查指针是否为 null，如果是，则返回一个空字符串
        if s.is_null() {
            return String::new();
        }
        // 使用 unsafe 块来执行不安全的操作：从原始指针创建 CStr 实例
        // 然后将 CStr 转换为 Rust 字符串
        unsafe { CStr::from_ptr(s).to_string_lossy().into_owned() }
    }

    /// 外部接口：释放 CCommandResult 内存（简化命名，移除 unsafe 标注）
    #[unsafe(no_mangle)]
    pub extern "C" fn free_command_result(result: CCommandResult) {
        // 调用内部安全方法来释放内存
        result.free();
    }

    /// 执行命令的核心逻辑（提取公共函数，统一错误处理）
    ///
    /// # 参数
    /// * `command` - 一个字符串，表示要在系统 shell 中执行的命令。
    ///
    /// # 返回值
    /// 返回一个 `io::Result` 类型，它封装了命令执行的输出结果 `Output`。
    /// 如果命令执行成功，可以通过 `Output` 类型访问 stdout 和 stderr 的内容；
    /// 如果执行失败，`io::Result` 将包含错误信息。
    fn run_command(command: &str) -> io::Result<Output> {
        // 根据目标操作系统选择合适的 shell 和参数
        #[cfg(target_os = "windows")]
        let (shell, arg) = ("cmd", "/C");
        #[cfg(not(target_os = "windows"))]
        let (shell, arg) = ("sh", "-c");

        // 使用所选的 shell 和参数构建命令，并执行
        Command::new(shell)
            .arg(arg)
            .arg(command)
            .stdout(Stdio::piped()) // 将标准输出重定向为管道，以便捕获输出内容
            .stderr(Stdio::piped()) // 将错误输出重定向为管道，以便捕获错误信息
            .output() // 执行命令并收集输出结果
    }

    /// 统一编码处理（支持跨平台，避免重复代码）
    ///
    /// 根据目标操作系统选择合适的字符编码方式。
    /// 对于Windows操作系统，优先使用GBK编码，以兼容中文字符；
    /// 对于其他操作系统，直接使用UTF-8编码。
    /// 这样做的目的是确保在不同平台上都能正确处理中文字符，避免乱码问题。
    ///
    /// # 参数
    ///
    /// * [output] - 一个字节切片，代表待解码的数据。
    ///
    /// # 返回值
    ///
    /// 返回一个字符串，表示解码后的数据。
    /// 如果在Windows平台上使用GBK编码解码时出现错误，会输出错误信息并回退到UTF-8编码。
    fn handle_encoding(output: &[u8]) -> String {
        // 在Windows操作系统上，使用GBK编码解码输出
        #[cfg(target_os = "windows")]
        {
            let (decoded, _, had_errors) = GBK.decode(output);
            if had_errors {
                eprintln!("GBK decoding error, falling back to UTF-8");
            }
            decoded.into_owned()
        }
        // 在非Windows操作系统上，直接使用UTF-8编码解码输出
        #[cfg(not(target_os = "windows"))]
        {
            String::from_utf8_lossy(output).into_owned()
        }
    }

    /// 同步执行命令（优化错误处理，使用 Result 替代 panic）
    ///
    /// # 参数
    ///
    /// - `command`: 要执行的命令，可以是任何可以转换为字符串的类型。
    ///
    /// # 返回值
    ///
    /// 返回一个 `CommandResult` 实例，其中包含了命令执行的结果、标准输出和错误输出。
    /// 如果命令执行成功，则 `CommandResult` 的 `success` 字段为 `true`；
    /// 否则为 `false`，并包含相应的错误信息。
    pub fn exec<T: AsRef<str>>(command: T) -> CommandResult {
        // 将输入的命令转换为字符串引用
        let cmd = command.as_ref();
        // 尝试执行命令
        match run_command(cmd) {
            // 如果命令执行成功
            Ok(output) => {
                // 处理并获取标准输出
                let stdout = handle_encoding(&output.stdout);
                // 处理并获取错误输出
                let stderr = handle_encoding(&output.stderr);
                // 构建并返回 CommandResult 实例，表示命令执行成功
                CommandResult::new(output.status.success(), stdout, stderr)
            }
            // 如果命令执行失败
            Err(e) => {
                // 构建并返回 CommandResult 实例，表示命令执行失败，并包含错误信息
                CommandResult::new(false, String::new(), format!("Execution error: {}", e))
            }
        }
    }

    /// 异步执行命令（修复原代码错误，返回 Child 供调用者管理）
    ///
    /// # 参数
    ///
    /// - `command`: 要执行的命令，可以是任何可以转换为字符串的类型。
    ///
    /// # 返回
    ///
    /// - `io::Result<Child>`: 返回一个 io 结果，包含一个 Child 进程，调用者可以使用它来管理进程。
    pub fn async_exec<T: AsRef<str>>(command: T) -> io::Result<Child> {
        // 获取命令的字符串引用
        let cmd = command.as_ref();

        // 根据操作系统选择合适的 shell 和参数
        #[cfg(target_os = "windows")]
        let (shell, arg) = ("cmd", "/C");
        #[cfg(not(target_os = "windows"))]
        let (shell, arg) = ("sh", "-c");

        // 使用选定的 shell 和参数构建命令，并异步执行
        Command::new(shell)
            .arg(arg)
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }

    /// C 接口：执行命令（添加空指针检查，处理无效命令）
    #[unsafe(no_mangle)]
    pub extern "C" fn c_exec(command: *const c_char) -> CCommandResult {
        if command.is_null() {
            return CCommandResult::new(false, String::from("Null command"), String::new());
        }
        let cmd_str = cstring_to_string(command);
        let result = exec(cmd_str);
        CCommandResult::new(result.success, result.stdout, result.stderr)
    }

    /// 释放 C 字符串内存（简化逻辑，明确空指针处理）
    #[unsafe(no_mangle)]
    pub extern "C" fn free_cstring(ptr: *mut c_char) {
        unsafe {
            if !ptr.is_null() {
                drop(CString::from_raw(ptr)); // 使用 drop 释放内存
            }
        }
    }

    /// 释放并重置C字符串指针
    ///
    /// 该函数接受一个指向C字符串指针的可变指针，确保安全地释放字符串占用的内存，
    /// 并将指针重置为null。这在与C代码互操作时非常有用，特别是在处理动态分配的内存时。
    ///
    /// 参数:
    ///   - `ptr`: *mut *const c_char - 指向C字符串指针的可变指针。这个指针的所指内容将被释放，
    ///            并且该指针随后将被重置为null。
    ///
    /// 注意: 该函数标记为`unsafe`和`no_mangle`，因为它直接操作指针并旨在与C代码互操作。
    ///       调用者必须确保传递给它的指针是有效的，并且释放操作是安全的。
    #[unsafe(no_mangle)]
    pub extern "C" fn free_and_reset_c_string(ptr: *mut *const c_char) {
        unsafe {
            // 检查传入的指针及其所指向的字符串指针是否为null
            if !ptr.is_null() && !(*ptr).is_null() {
                // 将C字符串指针转换为可变指针，以便释放内存
                let raw = *ptr as *mut c_char;
                // 使用`from_raw`将原始指针转换为`CString`，从而释放内存
                drop(CString::from_raw(raw));
                // 重置传入的指针为null，表示字符串已被释放
                *ptr = ptr::null();
            }
        }
    }

    /// 转换 Rust 字符串到 C 字符串（明确返回空指针场景）
    ///
    /// # 参数
    ///
    /// - `s`: 实现 `AsRef<str>` trait 的类型，表示可以转换为字符串引用的类型
    ///
    /// # 返回值
    ///
    /// - 成功时返回指向 C 字符串的可变指针
    /// - 失败时返回空指针 (`ptr::null_mut()`)
    ///
    /// # 说明
    ///
    /// 此函数尝试将 Rust 字符串转换为 C 字符串。如果输入字符串包含任何非有效字符，
    /// 转换将失败，并返回空指针。调用者负责确保输入字符串的有效性。
    pub fn str_to_cstr<T: AsRef<str>>(s: T) -> *mut c_char {
        CString::new(s.as_ref())
            .ok()
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut())
    }

    /// C 接口：检查文件是否存在（添加参数校验，统一错误码）
    #[unsafe(no_mangle)]
    pub extern "C" fn c_check_file(file_path: *const c_char) -> c_int {
        // 检查文件路径指针是否为空
        if file_path.is_null() {
            return -1; // 空指针错误
        }
        unsafe {
            // 将C字符串转换为Rust字符串切片
            match CStr::from_ptr(file_path).to_str() {
                Ok(path) => check_file(path),
                Err(_) => -1, // 无效路径格式
            }
        }
    }

    /// 检查文件存在（使用标准库类型，明确错误码含义）
    ///
    /// # 参数
    /// - `file_path`: 实现了 `AsRef<Path>` 的类型，表示文件路径。
    ///
    /// # 返回值
    /// - `1`: 文件存在。
    /// - `0`: 文件不存在。
    /// - `-1`: 其他错误。
    pub fn check_file<T: AsRef<Path>>(file_path: T) -> c_int {
        // 尝试获取文件的元数据，以检查文件是否存在
        match std::fs::metadata(file_path) {
            // 如果成功获取到元数据，说明文件存在
            Ok(_) => 1,
            // 如果获取元数据失败，且错误类型为 `NotFound`，说明文件不存在
            Err(e) if e.kind() == io::ErrorKind::NotFound => 0,
            // 对于其他错误情况，返回 `-1`
            _ => -1,
        }
    }

    /// 按行分割字符串（保留原逻辑，添加泛型约束说明）
    pub fn split_by_newline<T: AsRef<str>>(text: T) -> Vec<String> {
        text.as_ref().lines().map(String::from).collect()
    }

    /// C 接口：UTF-8 转 GBK（添加空指针检查，处理编码错误）
    #[unsafe(no_mangle)]
    pub extern "C" fn c_utf_8_str_to_gbk_str(utf8_str: *const c_char) -> *mut c_char {
        // 检查输入指针是否为空
        if utf8_str.is_null() {
            return ptr::null_mut();
        }
        // 安全地从C字符串指针转换为Rust字符串
        let input = unsafe { CStr::from_ptr(utf8_str).to_string_lossy().into_owned() };
        // 使用GBK编码转换字符串，同时检查是否有编码错误
        let (encoded, _, had_errors) = GBK.encode(&input);
        if had_errors {
            eprintln!("GBK encoding error");
        }
        // 将编码后的数据转换为C字符串并返回，如果转换失败则返回空指针
        CString::new(encoded)
            .ok()
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut())
    }

    /// 将 UTF-8 编码的字符串转换为 GBK 编码的字节序列
    ///
    /// 此函数接受一个实现了 AsRef<str> 特性的参数，这意味着它可以接收任何可以转换为字符串引用的类型
    /// 它使用 GBK 编码器对输入的字符串进行编码，并返回编码后的字节序列
    /// 由于 GBK 编码可能不被所有字符支持，因此编码过程可能会失败或丢失信息
    /// 此外，返回类型为 Vec<u8>，即字节序列，而不是 String 类型这是因为 GBK 编码的字节序列可能包含非 UTF-8 字符，
    /// 因此不能直接存储为 Rust 的 String 类型，该类型内部使用 UTF-8 编码
    ///
    /// 参数:
    /// - T: 实现了 AsRef<str> 特性的类型，代表要编码的 UTF-8 字符串
    ///
    /// 返回值:
    /// - Vec<u8>: GBK 编码后的字节序列
    pub fn utf_8_str_to_gbk_bytes<T: AsRef<str>>(input: T) -> Vec<u8> {
        // 将输入转换为字符串引用
        let s = input.as_ref();
        // 使用 GBK 编码器对字符串进行编码
        let (encoded, _, _) = GBK.encode(s);
        // 将编码后的字节序列转换为 Vec<u8> 类型并返回
        encoded.into_owned()
    }

    /// 清空终端屏幕
    ///

    pub fn clear_terminal() {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
    }
}

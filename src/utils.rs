pub mod utils {
    use std::ffi::{c_char, CStr, CString};
    use std::process::{Command, Stdio};
    use std::sync::{mpsc, Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{fs, ptr, thread};
    use std::marker::PhantomData;
    use std::os::windows::process::CommandExt;
    use std::sync::mpsc::{Receiver, Sender};

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
        // 将 C 风格字符串转换为 Rust 字符串
        let command_str = cstring_to_string(command);
        let com = if cfg!(target_os = "windows") {
            format!("chcp 65001 >nul && set LANG=en_US.UTF-8 && {}", command_str)
        } else {
            format!("export LANG=en_US.UTF-8 && {}", command_str)
        };

        // 根据目标操作系统选择合适的 shell 命令
        #[cfg(target_os = "windows")]
        let shell_command = "cmd";
        #[cfg(not(target_os = "windows"))]
        let shell_command = "sh"; // 注意这里改为 'sh' 而不是 'bin/bash'

        // 根据目标操作系统选择合适的命令参数前缀
        #[cfg(target_os = "windows")]
        let arg_prefix = "/C";
        #[cfg(not(target_os = "windows"))]
        let arg_prefix = "-c";

        // 执行命令并获取输出和错误信息
        let output = match Command::new(shell_command)
            .arg(arg_prefix) // 传递参数前缀
            .arg(&com) // 传递命令字符串
            .creation_flags(0x08000000) // Windows only flag
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(output) => output,
            Err(_) => return CommandResult::new(false, ptr::null_mut(), ptr::null_mut()),
        };

        // 将标准输出转换为 C 兼容的字符串
        let stdout_str = String::from_utf8_lossy(&output.stdout).into_owned();
        let stdout_cstring = CString::new(stdout_str).unwrap_or_else(|_| CString::new("").unwrap());
        let stdout_ptr = stdout_cstring.into_raw();

        // 将标准错误转换为 C 兼容的字符串
        let stderr_str = String::from_utf8_lossy(&output.stderr).into_owned();
        let stderr_cstring = CString::new(stderr_str).unwrap_or_else(|_| CString::new("").unwrap());
        let stderr_ptr = stderr_cstring.into_raw();

        // 创建并返回命令执行结果
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


    pub fn str_to_cstr(s: String) -> *mut c_char {
        // 创建一个新的 C 风格字符串
        match CString::new(s) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => ptr::null_mut(), // 或者选择其他方式处理错误
        }
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
        let file_path_str = cstring_to_string(file_path);
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

    struct SetConsoleOutputCP(i32);

    #[no_mangle]
#[cfg(target_os = "windows")]
/// 设置控制台输出代码页为UTF-8 (代码页65001)。
///
/// 此函数用于确保控制台输出使用UTF-8编码，从而正确显示多语言字符。
/// 仅在Windows操作系统上可用，并通过调用Windows API `SetConsoleOutputCP` 实现。
///
/// # 安全性
/// 该函数使用了`unsafe`块来调用外部的Windows API。由于API调用本身是安全的，
/// 并且没有涉及指针操作或其他不安全行为，因此这里的`unsafe`主要是为了遵循API调用的约定。
///
/// # 注意事项
/// - 仅在目标操作系统为Windows时编译此函数。
/// - 此函数被标记为`#[no_mangle]`以防止符号名在链接时被修改。
pub extern "C" fn set_console_output_cp_to_utf8() {
    unsafe {
        // 声明外部Windows API函数
        extern "system" {
            fn SetConsoleOutputCP(codepage: u32) -> u32;
        }

        // 调用原始的SetConsoleOutputCP函数，设置为UTF-8 (代码页65001)
        SetConsoleOutputCP(65001);
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

    pub fn split_by_newline(text: &str) -> Vec<String> {
        // 使用lines方法按行分隔字符串，然后使用map方法将每一行转换为String类型，最后收集到一个Vec中
        text.lines().map(String::from).collect()
    }
    type Job<R> = Box<dyn FnOnce() -> R + Send>;

    enum Message<R> {
        NewJob(Job<R>, Sender<R>),
        Terminate,
    }

    struct Worker<T, R> {
        id: usize,
        thread: Option<thread::JoinHandle<()>>,
        _phantom: PhantomData<(T, R)>,
    }

    impl<T, R> Worker<T, R>
    where
        T: Send + 'static,
        R: Send + 'static,
    {
        fn new(id: usize, receiver: Arc<Mutex<Receiver<Message<R>>>>) -> Worker<T, R> {
            let thread = thread::spawn(move || loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job, tx) => {
                        let result = job();
                        tx.send(result).expect("Failed to send result");
                    },
                    Message::Terminate => break,
                }
            });

            Worker {
                id,
                thread: Some(thread),
                _phantom: PhantomData,
            }
        }
    }

    pub struct ThreadPool<T, R> {
        workers: Vec<Worker<T, R>>,
        sender: Sender<Message<R>>,
    }

    impl<T, R> ThreadPool<T, R>
    where
        T: Send + 'static,
        R: Send + 'static,
    {
        pub fn new(size: usize) -> ThreadPool<T, R> {
            assert!(size > 0);

            let (sender, receiver) = mpsc::channel();
            let receiver = Arc::new(Mutex::new(receiver));
            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }

            ThreadPool { workers, sender }
        }

        pub fn submit<F>(&self, task: F, arg: T) -> Receiver<R>
        where
            F: FnOnce(T) -> R + Send + 'static,
        {
            let (tx, rx) = mpsc::channel();
            // 创建一个新的闭包，该闭包捕获了 `task` 和 `arg`
            let job = Box::new(move || task(arg));
            self.sender.send(Message::NewJob(job, tx)).unwrap();
            rx
        }
    }

    impl<T, R> Drop for ThreadPool<T, R> {
        fn drop(&mut self) {
            for _ in &mut self.workers {
                self.sender.send(Message::Terminate).unwrap();
            }

            for worker in &mut self.workers {
                if let Some(thread) = worker.thread.take() {
                    thread.join().unwrap();
                }
            }
        }
    }}
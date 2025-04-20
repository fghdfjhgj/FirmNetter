// src/lib.rs
pub mod web {
    use crossbeam::queue::ArrayQueue;
    use memmap2::MmapMut;
    use once_cell::sync::Lazy;
    use rayon::iter::IntoParallelRefIterator;
    use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
    use reqwest::blocking::Client;
    use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_TYPE};
    use serde::Serialize;
    use std::collections::HashMap;
    use std::ffi::{c_char, CStr, CString};
    use std::fmt;
    use std::fs::{metadata, rename, OpenOptions};
    use std::io::Read;
    use std::path::Path;
    use std::ptr;
    use std::time::Duration;
    use rayon::iter::IndexedParallelIterator;

    // 全局HTTP客户端（复用连接池）
    static GLOBAL_CLIENT: Lazy<Client> = Lazy::new(|| {
        Client::builder()
            .pool_max_idle_per_host(20)
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap()
    });

    // 错误类型增强
    #[derive(Debug)]
    pub enum WebError {
        RequestError(reqwest::Error),
        Utf8Error(std::str::Utf8Error),
        Io(std::io::Error),
        Server(String),
        ValidationFailed,
        BufferPoolEmpty, // 添加这一行
        BufferPoolFull,
    }

    impl fmt::Display for WebError {
        /// 实现 `fmt` 方法以提供自定义错误类型的格式化输出。
///
/// # 参数
/// - `&self`: 当前错误类型的引用，用于匹配具体的错误类型。
/// - `f`: 一个可变的 `Formatter` 引用，用于构建格式化的字符串输出。
///
/// # 返回值
/// - `fmt::Result`: 表示格式化操作是否成功。如果成功，则返回 `Ok`；否则返回错误。
///
/// # 功能描述
/// 根据不同的错误类型，将错误信息格式化为字符串并输出。如果遇到未知的错误类型，则返回格式化错误。
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // 使用 match 语句匹配不同的错误类型，并调用 `write!` 宏将错误信息写入到 Formatter 中。
    match self {
        Self::RequestError(e) => write!(f, "Request error: {}", e), // 格式化请求错误
        Self::Utf8Error(e) => write!(f, "UTF-8 conversion error: {}", e), // 格式化 UTF-8 转换错误
        Self::Io(e) => write!(f, "IO error: {}", e), // 格式化 IO 错误
        Self::Server(e) => write!(f, "Server error: {}", e), // 格式化服务器错误
        Self::ValidationFailed => write!(f, "File validation failed"), // 格式化文件验证失败错误
        _ => {
            Err(fmt::Error) // 如果是未知错误类型，返回格式化错误
        }
    }
}

    }

    impl std::error::Error for WebError {}

    impl From<reqwest::Error> for WebError {
        /// 从reqwest::Error类型错误中创建WebError实例
        ///
        /// 此函数实现了From<reqwest::Error> for WebError的转换，允许reqwest::Error无缝转换为WebError类型
        /// 主要用于错误处理，当遇到网络请求错误时，可以统一错误类型，便于错误传播和处理
        ///
        /// 参数:
        /// - err: reqwest::Error类型，表示网络请求中发生的错误
        ///
        /// 返回值:
        /// 返回WebError::RequestError(err)实例，将输入的reqwest::Error包装为WebError枚举中的RequestError变体
        fn from(err: reqwest::Error) -> Self {
            WebError::RequestError(err)
        }
    }

    impl From<std::str::Utf8Error> for WebError {
        /// 从 `std::str::Utf8Error` 类型的错误中创建 `WebError` 实例。
        ///
        /// # 参数
        /// * `err` - 一个 `std::str::Utf8Error` 类型的错误，表示在处理 UTF-8 字符串时发生的错误。
        ///
        /// # 返回值
        /// 返回 `WebError` 枚举的 `Utf8Error` 变体，用于表示在 Web 相关操作中遇到的 UTF-8 编码错误。
        fn from(err: std::str::Utf8Error) -> Self {
            WebError::Utf8Error(err)
        }
    }

    impl From<std::io::Error> for WebError {
        /// 从`std::io::Error`类型错误转换为`WebError`类型错误
        ///
        /// # 参数
        /// * `err`: 标准库中的IO错误，表示在进行IO操作时发生的错误
        ///
        /// # 返回值
        /// 返回一个`WebError`枚举的`Io`变体实例，用于表示Web错误中的IO错误
        fn from(err: std::io::Error) -> Self {
            WebError::Io(err)
        }
    }

    // POST响应结构体
    #[derive(Debug)]
    pub struct ResPost {
        pub status_code: i32,
        pub body: ResponseBody,
    }

    // 响应体类型
    #[derive(Debug)]
    pub enum ResponseBody {
        Text(String),
        Bytes(Vec<u8>),
    }

    impl fmt::Display for ResponseBody {
        /// 格式化响应体以便显示
        ///
        /// 此方法实现了 Display trait，允许以字符串形式输出响应体内容
        /// 它根据响应体是文本还是字节数据来选择合适的格式化方式
        ///
        /// 参数:
        /// - f: 格式化器，用于输出格式化后的字符串
        ///
        /// 返回:
        /// - fmt::Result: 格式化操作的结果，表示操作是否成功
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            // 根据ResponseBody的类型，选择合适的格式化方式
            match self {
                // 如果是文本类型，则直接写入文本内容
                ResponseBody::Text(text) => write!(f, "{}", text),
                // 如果是字节类型，则尝试将其转换为UTF-8文本
                ResponseBody::Bytes(bytes) => {
                    // 如果转换成功，则写入文本内容
                    if let Ok(text) = std::str::from_utf8(bytes) {
                        write!(f, "{}", text)
                    } else {
                        // 如果转换失败，则写入提示信息，表明收到了非UTF-8的二进制数据
                        write!(f, "Received binary data that is not valid UTF-8")
                    }
                }
            }
        }
    }

    impl ResPost {
        /// 创建一个新的ResPost实例
        ///
        /// # 参数
        ///
        /// - `status_code`: HTTP状态码，表示API响应的状态
        /// - `body`: 响应体，包含API返回的数据
        ///
        /// # 返回值
        ///
        /// 返回一个新的ResPost实例，该实例包含提供的状态码和响应体
        pub fn new(status_code: i32, body: ResponseBody) -> ResPost {
            ResPost { status_code, body }
        }
    }

    // HTTP POST接口
    pub fn web_post<T, B>(
        url: T,
        body: B,
        way: bool,
        raw_bytes: bool,
    ) -> Result<ResPost, WebError>
    where
        T: reqwest::IntoUrl,
        B: Serialize,
    {
        // 根据way参数决定是发送JSON数据还是表单数据
        let response = if way {
            GLOBAL_CLIENT.post(url).json(&body).send()?
        } else {
            GLOBAL_CLIENT.post(url).form(&body).send()?
        };

        // 获取HTTP响应状态码
        let status_code = response.status().as_u16() as i32;
        // 获取响应的内容类型
        let content_type = response.headers()
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");

        // 根据raw_bytes参数决定是按文本还是按字节流处理响应体
        let res_body = if raw_bytes {
            ResponseBody::Bytes(response.bytes()?.to_vec())
        } else {
            // 根据内容类型决定是处理为文本还是字节流
            match content_type {
                t if t.contains("text/") || t.contains("json") =>
                    ResponseBody::Text(response.text()?),
                _ => ResponseBody::Bytes(response.bytes()?.to_vec())
            }
        };

        // 构造ResPost对象并返回
        Ok(ResPost::new(status_code, res_body))
    }

    // C接口结构体
    #[repr(C)]
    pub struct CResPost {
        pub status_code: i32,
        pub body_type: i32,
        pub body_text: *const c_char,
        pub body_bytes: *const u8,
        pub body_len: usize,
    }

    // 定义一个对外的C接口函数，用于执行POST请求
    // 该函数接收URL、表单数据的键值对、结果容器、请求方式和是否返回原始字节的标志
    // 返回一个整数表示操作是否成功
    #[no_mangle]
    pub extern "C" fn c_web_post(
        url: *const c_char,
        form_data_keys: *const *const c_char,
        form_data_values: *const *const c_char,
        form_data_count: usize,
        result: *mut CResPost,
        way: bool,
        raw_bytes: bool,
    ) -> i32 {
        // 将C类型的字符串和数据结构转换为Rust可以处理的形式
        unsafe {
            // 转换URL为Rust字符串
            let url_str = match CStr::from_ptr(url).to_str() {
                Ok(s) => s,
                Err(_) => return 1,
            };

            // 将C类型的数组转换为Rust切片
            let keys = std::slice::from_raw_parts(form_data_keys, form_data_count);
            let values = std::slice::from_raw_parts(form_data_values, form_data_count);

            // 构建表单数据的HashMap
            let mut form_data = HashMap::with_capacity(form_data_count);
            for (k, v) in keys.iter().zip(values.iter()) {
                let key = match CStr::from_ptr(*k).to_str() {
                    Ok(s) => s,
                    Err(_) => return 1,
                };
                let value = match CStr::from_ptr(*v).to_str() {
                    Ok(s) => s,
                    Err(_) => return 1,
                };
                form_data.insert(key.to_owned(), value.to_owned());
            }

            // 调用web_post函数执行POST请求，并根据结果填充CResPost结构体
            match web_post(url_str, &form_data, way, raw_bytes) {
                Ok(res_post) => {
                    let result_ref = &mut *result;
                    result_ref.status_code = res_post.status_code;

                    // 根据响应体的类型，填充CResPost结构体的相应字段
                    match res_post.body {
                        ResponseBody::Text(text) => {
                            result_ref.body_type = 0;
                            let c_str = CString::new(text).unwrap_or_else(|_| CString::new("Invalid UTF-8").unwrap());
                            result_ref.body_text = c_str.into_raw();
                            result_ref.body_bytes = ptr::null();
                            result_ref.body_len = 0;
                        }
                        ResponseBody::Bytes(bytes) => {
                            result_ref.body_type = 1;
                            result_ref.body_text = ptr::null();
                            result_ref.body_bytes = bytes.as_ptr();
                            result_ref.body_len = bytes.len();
                        }
                    }
                    0
                }
                Err(_) => 1,
            }
        }
    }

    /// 释放C字符串
    ///
    /// # Safety
    /// 此函数被标记为unsafe，因为调用者必须确保传递的指针是有效的，并且未被同时释放或访问。
    /// 此函数被标记为no_mangle，以防止名称修饰，使其可以从C代码中调用。
    /// # Parameters
    /// * `s`: *mut c_char - 指向要释放的C字符串的指针。如果指针为NULL，函数将不执行任何操作。
    /// # Returns
    /// 无返回值。该函数释放内存后，指针s应被视为无效。
    #[unsafe(no_mangle)]
    pub extern "C" fn free_c_string(s: *mut c_char) {
        unsafe {
            // 检查指针是否为NULL
            if !s.is_null() {
                // 从原始指针创建CString，这将接管指针的所有权并释放内存
                let _ = CString::from_raw(s);
            }
        }
    }

    // 下载结果简化版
    #[derive(Debug)]
    pub struct DownloadResult {
        pub threads_used: usize,
        pub save_path: String,
        pub file_name: String,
    }

    /// 下载文件的优化实现，支持单线程和多线程下载。
///
/// # 参数
/// - `url`: 文件的下载地址，可以是任何实现了 `AsRef<str>` 的类型。
/// - `save_path`: 文件保存的目标路径，可以是任何实现了 `AsRef<Path>` 的类型。
/// - `requested_threads`: 请求的线程数，实际使用的线程数会根据文件大小和系统资源进行调整。
/// - `buffer_pool`: 缓冲区池的引用，用于管理下载过程中使用的缓冲区。
///
/// # 返回值
/// - 成功时返回 `DownloadResult`，包含实际使用的线程数、保存路径和文件名。
/// - 失败时返回 `WebError`，表示下载过程中发生的错误。
///
/// # 功能描述
/// 该函数通过 HTTP 协议下载指定 URL 的文件，并将其保存到指定路径。支持单线程和多线程下载模式，
/// 根据服务器是否支持分块下载（`Accept-Ranges: bytes`）来决定使用哪种模式。
///
/// 如果服务器不支持分块下载，则使用单线程下载；如果支持，则根据文件大小和请求的线程数计算最优线程数，
/// 并将文件分为多个块进行并行下载。
///
/// # 注意事项
/// - 下载过程中会创建一个临时文件，下载完成后将其重命名为目标文件。
/// - 使用内存映射技术提高文件写入效率。
/// - 下载完成后会验证文件大小，确保下载内容完整。
pub fn download_file<T: AsRef<str>, P: AsRef<Path>>(
    url: T,
    save_path: P,
    requested_threads: usize,
    buffer_pool: &BufferPool,
) -> Result<DownloadResult, WebError> {
    let url = url.as_ref();
    let original_path = save_path.as_ref().to_path_buf();
    let temp_path = original_path.with_extension("download");

    // 获取文件信息，包括是否支持分块下载和文件总大小
    let response = GLOBAL_CLIENT.head(url).send()?;
    let supports_chunked = response.headers()
        .get(ACCEPT_RANGES)
        .map_or(false, |v| v == "bytes");
    let total_size = response.headers()
        .get(CONTENT_LENGTH)
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| ct.parse().ok())
        .ok_or(WebError::Server("Missing Content-Length".into()))?;

    // 创建临时文件并设置其大小为文件总大小
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&temp_path)?;
    file.set_len(total_size)?;

    // 使用内存映射技术映射临时文件
    let mut mem_map = unsafe { MmapMut::map_mut(&file) }.map_err(WebError::from)?;

    // 如果服务器不支持分块下载，则使用单线程下载
    if !supports_chunked {
        let mut response = GLOBAL_CLIENT.get(url).send()?;
        let mut file = OpenOptions::new().write(true).open(&temp_path)?;
        std::io::copy(&mut response, &mut file)?;
        validate_file(&temp_path, total_size)?;
        rename(&temp_path, &original_path)?;

        let file_name = original_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_file")
            .to_string();
        let save_path = original_path.to_string_lossy().into_owned();

        return Ok(DownloadResult {
            threads_used: 1,
            save_path,
            file_name,
        });
    }

    // 计算最优线程数并生成分块信息
    let actual_threads = optimal_thread_count(requested_threads, total_size);
    let chunks = balanced_chunks(total_size, actual_threads);

    // 将内存映射按照分块信息分割成多个可变切片
    let mut slices: Vec<&mut [u8]> = Vec::with_capacity(chunks.len());
    let mut remaining_mem_map = &mut mem_map[..];
    for &(start, end) in &chunks {
        let (left, right) = remaining_mem_map.split_at_mut((end - start) as usize);
        slices.push(left);
        remaining_mem_map = right;
    }

    // 并行处理每个分块，从服务器下载数据并写入对应的切片
    chunks.par_iter().zip(slices.par_iter_mut()).try_for_each(|(&(start, end), slice)| {
        let client = GLOBAL_CLIENT.clone();
        let mut response = client.get(url)
            .header("Range", format!("bytes={}-{}", start, end))
            .send()?;

        let mut buffer = buffer_pool.get()?; // 从缓冲区池中获取缓冲区

        loop {
            let read = response.read(&mut buffer)?;
            if read == 0 { break; }
            slice[0..read].copy_from_slice(&buffer[..read]);
        }

        buffer_pool.put(buffer)?; // 将缓冲区归还到池中
        Ok::<(), WebError>(())
    })?;

    // 确保内存映射的内容写入磁盘
    mem_map.flush().map_err(WebError::from)?;

    // 验证文件大小并重命名临时文件为目标文件
    validate_file(&temp_path, total_size)?;
    rename(&temp_path, &original_path)?;

    let file_name = original_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown_file")
        .to_string();
    let save_path = original_path.to_string_lossy().into_owned();

    Ok(DownloadResult {
        threads_used: actual_threads,
        save_path,
        file_name,
    })
}


    // 缓冲区池实现
    pub struct BufferPool {
        pool: ArrayQueue<Vec<u8>>,
        buffer_size: usize,
    }

    impl BufferPool {
        /// 从缓冲区池中获取一个缓冲区
        ///
        /// 此方法尝试从缓冲区池中弹出一个缓冲区如果池中没有可用的缓冲区，
        /// 它将创建一个新的缓冲区并返回
        ///
        /// 返回:
        /// - `Result<Vec<u8>, WebError>`: 如果成功获取或创建缓冲区，则返回`Ok(Vec<u8>)`，
        ///   否则返回`Err(WebError)`表示错误
        pub fn get(&self) -> Result<Vec<u8>, WebError> {
            // 尝试从缓冲区池中弹出一个缓冲区
            if let Some(buf) = self.pool.pop() {
                Ok(buf)
            } else {
                // 如果缓冲区池为空，创建一个新的缓冲区
                Ok(vec![0; self.buffer_size])
            }
        }

        /// 将缓冲区放入池中
        ///
        /// 此函数从当前实例的缓冲区池中获取一个缓冲区，清空其内容并调整其大小，
        /// 然后尝试将其放回池中以供将来使用。如果池已满，此操作将失败。
        ///
        /// # 参数
        ///
        /// * `mut buf: Vec<u8>` - 一个可变的字节向量，代表要放入池中的缓冲区。
        ///   函数将清除这个缓冲区的内容并调整其大小，以匹配实例的缓冲区大小。
        ///
        /// # 返回
        ///
        /// * `Ok(())` - 如果缓冲区成功放入池中。
        /// * `Err(WebError::BufferPoolFull)` - 如果池已满，无法放入缓冲区。
        pub fn put(&self, mut buf: Vec<u8>) -> Result<(), WebError> {
            // 清空缓冲区内容，准备重新使用
            buf.clear();
            // 调整缓冲区大小，以匹配实例的缓冲区大小设置
            buf.resize(self.buffer_size, 0);
            // 尝试将缓冲区放回池中，如果池已满，则返回错误
            self.pool.push(buf).map_err(|_| WebError::BufferPoolFull)
        }
    }

    /// 计算最优线程数量
    ///
    /// 该函数旨在根据请求的线程数和数据总量，计算出最优的线程数量。
    /// 它通过比较基于CPU核心数和基于数据量的线程数，来确定最终的线程数量。
    ///
    /// # 参数
    ///
    /// - `requested`: 用户请求的线程数量。
    /// - `total`: 数据总量，用于计算基于数据量的线程数量。
    ///
    /// # 返回值
    ///
    /// 返回计算出的最优线程数量。
    fn optimal_thread_count(requested: usize, total: u64) -> usize {
        // 获取当前系统中的CPU核心数
        let cpu_cores = rayon::current_num_threads();

        // 根据数据总量计算理想的线程数量，每10MB数据分配一个线程
        let size_based = (total / (1024 * 1024 * 10)) as usize;

        // 最终的线程数量为请求的线程数量与CPU核心数和基于数据量计算出的线程数量的最小值之间的较大者，
        // 并且至少为1
        requested.clamp(1, cpu_cores.min(size_based).max(1))
    }

    const MIN_CHUNK_SIZE: u64 = 1024 * 1024; // 最小分块大小为 1MB

    fn balanced_chunks(total: u64, threads: usize) -> Vec<(u64, u64)> {
        // 初始化一个容量为 threads 的向量 chunks，用于存储每个分片的起始和结束位置
        let mut chunks = Vec::with_capacity(threads);
        // 初始化剩余未分配的数据大小为 total
        let mut remaining = total;
        // 初始化分片的起始位置为 0
        let mut start = 0;

        // 遍历每个线程，为每个线程计算并分配数据分片
        for i in 0..threads {
            // 计算当前分片的大小
            // 如果是最后一个分片，则包含所有剩余的数据
            // 否则，将剩余数据平均分配给未分配的线程，同时确保分片大小至少为 MIN_CHUNK_SIZE
            let chunk_size = if i == threads - 1 {
                remaining // 最后一个分块包含剩余的所有数据
            } else {
                (remaining / (threads - i) as u64).max(MIN_CHUNK_SIZE)
            };

            // 计算并设置当前分片的结束位置
            let end = start + chunk_size - 1;
            // 将当前分片的起始和结束位置添加到 chunks 向量中
            chunks.push((start, end));
            // 更新下一个分片的起始位置
            start += chunk_size;
            // 更新剩余未分配的数据大小
            remaining -= chunk_size;
        }

        // 返回包含所有分片范围的向量
        chunks
    }

    /// 验证文件大小是否与预期相符
    ///
    /// 此函数通过比较文件的实际大小与预期大小来验证文件是否符合预期
    /// 它用于确保文件的完整性或满足特定的文件大小要求
    ///
    /// # 参数
    ///
    /// * `path`: 一个指向文件的路径引用
    /// * `expected`: 一个 u64 类型的整数，表示预期的文件大小（以字节为单位）
    ///
    /// # 返回值
    ///
    /// * `Result<(), WebError>`: 如果文件大小与预期相符，则返回 Ok(())；
    ///   否则，返回 Err(WebError::ValidationFailed)，表示验证失败
    fn validate_file(path: &Path, expected: u64) -> Result<(), WebError> {
        // 获取文件的实际大小
        let actual = metadata(path)?.len();
        // 比较实际大小与预期大小
        if actual != expected {
            // 如果不相符，返回验证失败的错误
            Err(WebError::ValidationFailed)
        } else {
            // 如果相符，返回 Ok
            Ok(())
        }
    }
}
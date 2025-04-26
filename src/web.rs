// src/lib.rs
pub mod web {
    use crossbeam::queue::ArrayQueue;
    use memmap2::MmapMut;
    use once_cell::sync::Lazy;
    use percent_encoding::percent_decode_str;
    use rayon::iter::IndexedParallelIterator;
    use rayon::iter::IntoParallelRefIterator;
    use rayon::prelude::IntoParallelRefMutIterator;
    use rayon::prelude::*;
    use reqwest::Url;
    use reqwest::blocking::Client;
    use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_TYPE};
    use serde::Serialize;
    use std::collections::HashMap;
    use std::ffi::{CStr, CString, c_char};
    use std::fmt;
    use std::fs::{OpenOptions, metadata, rename};
    use std::io::Read;
    use std::path::Path;
    use std::ptr;
    use std::time::Duration;

    // 全局HTTP客户端
    static GLOBAL_CLIENT: Lazy<Client> = Lazy::new(|| {
        Client::builder()
            .pool_max_idle_per_host(20)
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap()
    });

    // 增强错误类型
    #[derive(Debug)]
    pub enum WebError {
        RequestError(reqwest::Error),
        Utf8Error(std::str::Utf8Error),
        Io(std::io::Error),
        Server(String),
        ValidationFailed,
        BufferPoolEmpty,
        BufferPoolFull,
    }

    impl fmt::Display for WebError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::RequestError(e) => write!(f, "Request error: {}", e),
                Self::Utf8Error(e) => write!(f, "UTF-8 conversion error: {}", e),
                Self::Io(e) => write!(f, "IO error: {}", e),
                Self::Server(e) => write!(f, "Server error: {}", e),
                Self::ValidationFailed => write!(f, "File validation failed"),
                Self::BufferPoolEmpty => write!(f, "Buffer pool is empty"),
                Self::BufferPoolFull => write!(f, "Buffer pool is full"),
            }
        }
    }

    impl std::error::Error for WebError {}

    // 错误转换实现
    impl From<reqwest::Error> for WebError {
        fn from(err: reqwest::Error) -> Self {
            WebError::RequestError(err)
        }
    }

    impl From<std::str::Utf8Error> for WebError {
        fn from(err: std::str::Utf8Error) -> Self {
            WebError::Utf8Error(err)
        }
    }

    impl From<std::io::Error> for WebError {
        fn from(err: std::io::Error) -> Self {
            WebError::Io(err)
        }
    }

    // POST响应结构
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
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                ResponseBody::Text(text) => write!(f, "{}", text),
                ResponseBody::Bytes(bytes) => {
                    if let Ok(text) = std::str::from_utf8(bytes) {
                        write!(f, "{}", text)
                    } else {
                        write!(f, "[Binary data ({} bytes)]", bytes.len())
                    }
                }
            }
        }
    }

    impl ResPost {
        pub fn new(status_code: i32, body: ResponseBody) -> ResPost {
            ResPost { status_code, body }
        }
    }

    // HTTP POST接口
    pub fn web_post<T, B>(url: T, body: B, way: bool, raw_bytes: bool) -> Result<ResPost, WebError>
    where
        T: reqwest::IntoUrl,
        B: Serialize,
    {
        let response = if way {
            GLOBAL_CLIENT.post(url).json(&body).send()?
        } else {
            GLOBAL_CLIENT.post(url).form(&body).send()?
        };

        let status_code = response.status().as_u16() as i32;
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");

        let res_body = if raw_bytes {
            ResponseBody::Bytes(response.bytes()?.to_vec())
        } else {
            match content_type {
                t if t.contains("text/") || t.contains("json") => {
                    ResponseBody::Text(response.text()?)
                }
                _ => ResponseBody::Bytes(response.bytes()?.to_vec()),
            }
        };

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

    // 修复后的FFI函数
    #[unsafe(no_mangle)]
    pub extern "C" fn c_web_post(
        url: *const c_char,
        form_data_keys: *const *const c_char,
        form_data_values: *const *const c_char,
        form_data_count: usize,
        result: *mut CResPost,
        way: bool,
        raw_bytes: bool,
    ) -> i32 {
        unsafe {
            let url_str = match CStr::from_ptr(url).to_str() {
                Ok(s) => s,
                Err(_) => return 1,
            };

            let keys = std::slice::from_raw_parts(form_data_keys, form_data_count);
            let values = std::slice::from_raw_parts(form_data_values, form_data_count);

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

            match web_post(url_str, &form_data, way, raw_bytes) {
                Ok(res_post) => {
                    let result_ref = &mut *result;
                    result_ref.status_code = res_post.status_code;

                    match res_post.body {
                        ResponseBody::Text(text) => {
                            result_ref.body_type = 0;
                            let c_str = CString::new(text)
                                .unwrap_or_else(|_| CString::new("Invalid UTF-8").unwrap());
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

    // 内存释放函数
    #[unsafe(no_mangle)]
    pub extern "C" fn free_c_string(s: *mut c_char) {
        unsafe {
            if !s.is_null() {
                let _ = CString::from_raw(s);
            }
        }
    }

    // 下载结果结构
    #[derive(Debug)]
    pub struct DownloadResult {
        pub threads_used: usize,
        pub save_path: String,
        pub file_name: String,
    }

    /// 下载文件的核心逻辑函数，支持单线程和多线程下载。
    ///
    /// # 参数
    /// - `url`: 文件的下载地址（字符串或可转换为字符串的类型）。
    /// - `save_path`: 文件保存的目标路径（路径或可转换为路径的类型）。
    /// - `requested_threads`: 用户请求的下载线程数。实际使用的线程数会根据文件大小和系统资源调整。
    /// - `mandatory_use`: 是否强制使用参数下载线程数。。
    /// - `buffer_pool`: 缓冲区池对象，用于复用内存缓冲区以提高性能。
    /// # 返回值
    /// - 成功时返回 `DownloadResult`，包含实际使用的线程数、保存路径和文件名。
    /// - 失败时返回 `WebError`，表示下载过程中发生的错误。
    ///
    /// # 功能描述
    /// 1. 检查服务器是否支持分块下载（通过 `Accept-Ranges` 头部判断）。
    /// 2. 获取文件总大小（通过 `Content-Length` 头部获取）。
    /// 3. 如果服务器不支持分块下载，则使用单线程下载整个文件。
    /// 4. 如果服务器支持分块下载，则计算最优线程数并划分下载块。
    /// 5. 使用多线程并发下载每个块，并将数据写入内存映射文件。
    /// 6. 下载完成后验证文件大小，并将临时文件重命名为目标文件。
    pub fn download_file<T: AsRef<str>, P: AsRef<Path>>(
        url: T,
        save_path: P,
        requested_threads: usize,
        mandatory_use: bool,
        buffer_pool: &BufferPool,
    ) -> Result<DownloadResult, WebError> {
        // 将输入参数转换为具体引用类型
        let url = url.as_ref();
        let mut original_path = save_path.as_ref().to_path_buf();

        // 如果目标路径是目录，自动从URL提取文件名
        if original_path.is_dir() {
            let file_name = extract_filename(url);
            original_path = original_path.join(file_name);
        }
        // 确保父目录存在
        if let Some(parent) = original_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 后续原有代码保持不变...
        let temp_path = original_path.with_extension("download");

        // 发送 HEAD 请求以获取文件元信息
        let response = GLOBAL_CLIENT.head(url).send()?;
        let supports_chunked = response
            .headers()
            .get(ACCEPT_RANGES)
            .map_or(false, |v| v == "bytes");
        let total_size = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|ct| ct.to_str().ok())
            .and_then(|ct| ct.parse().ok())
            .ok_or(WebError::Server("Missing Content-Length".into()))?;

        // 创建临时文件并设置文件大小
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&temp_path)?;
        file.set_len(total_size)?;

        // 映射文件到内存
        let mut mem_map = unsafe { MmapMut::map_mut(&file)? };

        // 如果服务器不支持分块下载，使用单线程下载
        if !supports_chunked {
            let mut response = GLOBAL_CLIENT.get(url).send()?;
            let mut file = OpenOptions::new().write(true).open(&temp_path)?;
            std::io::copy(&mut response, &mut file)?;
            validate_file(&temp_path, total_size)?;
            rename(&temp_path, &original_path)?;

            let file_name = original_path
                .file_name()
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
        let actual_threads: usize;
        // 计算最优线程数
        match mandatory_use {
            true => {
                actual_threads = requested_threads;
            }
            false => {
                actual_threads = optimal_thread_count(requested_threads, total_size);
            }
        }

        // 计算最优分块
        let chunks = balanced_chunks(total_size, actual_threads);

        // 为每个分块分配内存切片
        let mut slices: Vec<&mut [u8]> = Vec::with_capacity(chunks.len());
        let mut remaining_mem_map = &mut mem_map[..];
        for &(start, end) in &chunks {
            let chunk_size = (end - start + 1) as usize; // 计算包含end的字节数
            let (left, right) = remaining_mem_map.split_at_mut(chunk_size);
            slices.push(left);
            remaining_mem_map = right;
        }

        // 并发下载每个分块
        chunks.par_iter().zip(slices.par_iter_mut()).try_for_each(
            |((start, end), slice)| -> Result<(), WebError> {
                let client = GLOBAL_CLIENT.clone();
                let mut response = client
                    .get(url)
                    .header("Range", format!("bytes={}-{}", start, end))
                    .send()?;

                let mut buffer = buffer_pool.get()?;

                loop {
                    let read = response.read(&mut buffer)?;
                    if read == 0 {
                        break;
                    }
                    slice[0..read].copy_from_slice(&buffer[..read]);
                }

                buffer_pool.put(buffer)?;
                Ok(())
            },
        )?;

        // 刷新内存映射并验证文件完整性
        mem_map.flush()?;
        validate_file(&temp_path, total_size)?;
        rename(&temp_path, &original_path)?;

        // 构造返回结果
        let file_name = original_path
            .file_name()
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
        pub fn new(pool_size: usize, buffer_size: usize) -> Self {
            BufferPool {
                pool: ArrayQueue::new(pool_size),
                buffer_size,
            }
        }

        pub fn get(&self) -> Result<Vec<u8>, WebError> {
            if let Some(buf) = self.pool.pop() {
                Ok(buf)
            } else {
                Ok(vec![0; self.buffer_size])
            }
        }

        pub fn put(&self, mut buf: Vec<u8>) -> Result<(), WebError> {
            buf.clear();
            buf.resize(self.buffer_size, 0);
            self.pool.push(buf).map_err(|_| WebError::BufferPoolFull)
        }
    }

    /// 根据请求的线程数和总数据量，计算并返回最优的线程数
    ///
    /// 此函数旨在根据当前系统的CPU核心数和需要处理的数据量大小，平衡性能与资源消耗
    /// 它确保了所返回的线程数既不会因过小而无法充分利用系统资源，也不会因过大而导致资源过度消耗
    ///
    /// # 参数
    ///
    /// - `requested`: 用户请求的线程数这是建议的线程数量，函数会基于系统能力对此数值进行调整
    /// - `total`: 需要处理的总数据量（以字节为单位）这用于计算基于数据量的线程数
    ///
    /// # 返回值
    ///
    /// 返回一个`usize`类型的最优线程数，确保在系统CPU核心数和数据量之间取得平衡
    fn optimal_thread_count(requested: usize, total: u64) -> usize {
        // 获取当前系统中的CPU核心数，作为可用线程数的上限
        let cpu_cores = rayon::current_num_threads();

        // 根据总数据量计算一个初步的线程数建议值，每10MB数据建议使用一个线程
        let size_based = (total / (1024 * 1024 * 10)) as usize;

        // 最终的线程数选择：在1和CPU核心数与基于数据量的线程数建议值中较小的一个之间，选择最接近requested的值
        // 这样既保证了至少有一个线程被使用，也确保了不会超过系统的实际CPU核心数或数据量建议的线程数
        requested.clamp(1, cpu_cores.min(size_based).max(1))
    }

    const MIN_CHUNK_SIZE: u64 = 1024 * 1024;

    /// 根据总工作量和线程数，将工作量分成均衡的块
    /// 此函数旨在尽可能均匀地分配工作量，以优化多线程处理
    ///
    /// # 参数
    ///
    /// * `total` - 总工作量，以单位量表示
    /// * `threads` - 参与处理的线程数
    ///
    /// # 返回值
    ///
    /// 返回一个包含每个线程处理的工作量范围的向量元组
    /// 每个元组包含开始和结束位置，表示该线程应处理的工作量范围
    fn balanced_chunks(total: u64, threads: usize) -> Vec<(u64, u64)> {
        // 初始化容量为线程数的向量，用于存储每个线程的工作量范围
        let mut chunks = Vec::with_capacity(threads);
        // 初始化剩余工作量为总工作量
        let mut remaining = total;
        // 初始化起始位置为0
        let mut start = 0;

        // 遍历每个线程，分配工作量范围
        for i in 0..threads {
            // 计算当前线程的工作量大小
            // 如果是最后一个线程，分配所有剩余工作量
            // 否则，计算平均工作量，并确保最小工作量不小于预设的最小值
            let chunk_size = if i == threads - 1 {
                remaining
            } else {
                (remaining / (threads - i) as u64).max(MIN_CHUNK_SIZE)
            };

            // 计算当前线程工作量的结束位置
            let end = start + chunk_size - 1;
            // 将当前线程的工作量范围添加到向量中
            chunks.push((start, end));
            // 更新起始位置为下一个线程的工作量开始处
            start += chunk_size;
            // 更新剩余工作量
            remaining -= chunk_size;
        }

        // 返回每个线程的工作量范围
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
    /// * `expected`: 一个 u64 类型的整数，表示预期的文件大小
    ///
    /// # 返回值
    ///
    /// * `Result<(), WebError>`: 如果文件大小与预期相符，则返回 Ok(())；
    ///   否则，返回 Err(WebError::ValidationFailed) 表示验证失败
    fn validate_file(path: &Path, expected: u64) -> Result<(), WebError> {
        // 获取文件的实际大小
        let actual = metadata(path)?.len();
        // 比较实际大小与预期大小
        if actual != expected {
            // 如果不相符，返回验证失败的错误
            Err(WebError::ValidationFailed)
        } else {
            // 如果相符，返回 Ok 表示验证成功
            Ok(())
        }
    }
    /// 从给定的URL中提取文件名
    ///
    /// # 参数
    /// * `url` - 一个字符串切片，表示要解析的URL
    ///
    /// # 返回值
    /// 返回提取到的文件名字符串。如果URL无效或无法提取文件名，则返回默认的文件名
    pub fn extract_filename(url: &str) -> String {
        // 解析URL
        let parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return "invalid_url.bin".to_string(),
        };

        // 获取路径部分
        let path = parsed.path();

        // 处理空路径
        if path.is_empty() || path == "/" {
            return generate_default_name(&parsed);
        }

        // 分割路径段
        let segments: Vec<&str> = path.split('/').collect();

        // 查找最后一个有效段
        let filename = segments
            .iter()
            .rev()
            .find(|s| !s.is_empty())
            .map(|s| clean_filename(s))
            .unwrap_or_else(|| generate_default_name(&parsed));

        // 处理无扩展名
        add_default_extension(filename)
    }

    /// 清理文件名中的特殊字符和控制字符，以确保文件名在各个操作系统中的兼容性和安全性。
    ///
    /// # 参数
    /// * `raw`: &str - 原始的、可能包含特殊字符的文件名字符串。
    ///
    /// # 返回值
    /// 返回一个`String`，其中所有不安全的字符都被替换为下划线('_')。
    ///
    /// # 功能描述
    /// 本函数首先尝试对原始文件名进行URL解码，以处理可能的URL编码字符。
    /// 然后，它会移除或替换掉文件名中可能引起安全问题或在某些操作系统上不被允许的字符。
    fn clean_filename(raw: &str) -> String {
        // 尝试对原始字符串进行URL解码
        let decoded = percent_decode_str(raw).decode_utf8().unwrap_or_default();

        // 移除特殊字符
        decoded.replace(
            |c: char| {
                // 判断字符是否为控制字符，或属于文件名中的非法字符
                c.is_control() || c == '/' || c == '\\' || c == ':' || c == '*'
            },
            "_",
        )
    }

    fn generate_default_name(url: &Url) -> String {
        // 使用host作为基础名称
        let host = url.host_str().unwrap_or("unknown");
        format!("{}.bin", host)
    }

    fn add_default_extension(name: String) -> String {
        if name.contains('.') {
            name
        } else {
            format!("{}.bin", name)
        }
    }
    // C 结构体定义
    #[repr(C)]
    pub struct CDownloadResult {
        pub threads_used: usize,
        pub save_path: *const c_char,
        pub file_name: *const c_char,
    }

    // FFI 函数实现
    #[unsafe(no_mangle)]
    pub extern "C" fn c_download_file(
        url: *const c_char,
        save_path: *const c_char,
        requested_threads: usize,
        mandatory_use: bool,
        buffer_pool_size: usize,
        buffer_size: usize,
        result: *mut CDownloadResult,
    ) -> i32 {
        unsafe {
            let url_str = match CStr::from_ptr(url).to_str() {
                Ok(s) => s,
                Err(_) => return 1,
            };

            let save_path_str = match CStr::from_ptr(save_path).to_str() {
                Ok(s) => s,
                Err(_) => return 1,
            };

            let buffer_pool = BufferPool::new(buffer_pool_size, buffer_size);

            match download_file(
                url_str,
                save_path_str,
                requested_threads,
                mandatory_use,
                &buffer_pool,
            ) {
                Ok(download_result) => {
                    let result_ref = &mut *result;
                    result_ref.threads_used = download_result.threads_used;

                    let c_save_path = CString::new(download_result.save_path)
                        .unwrap_or_else(|_| CString::new("Invalid UTF-8").unwrap());
                    let c_file_name = CString::new(download_result.file_name)
                        .unwrap_or_else(|_| CString::new("Invalid UTF-8").unwrap());

                    result_ref.save_path = c_save_path.into_raw();
                    result_ref.file_name = c_file_name.into_raw();

                    0
                }
                Err(_) => 1,
            }
        }
    }
    #[test]
    fn test_download_file() {
        let url = "http://127.0.0.1:5244/d/home/sunyuze/work/home/sunyuze/work/sun/vite.config.ts?sign=FEprMHj6xCMfEEkcGSgt6Zw7nFtpKgqqpURTXHQfcSE=:0";
        let save_path = "/home/sunyuze/Downloads/";
        let threads = 4;
        let pool = BufferPool::new(10, 1024 * 1024);
        let res = download_file(url, save_path, threads, false, &pool);
        let save = res.unwrap().threads_used;
        println!("{}", save)
    }
}

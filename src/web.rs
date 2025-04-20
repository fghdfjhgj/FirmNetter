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
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::RequestError(e) => write!(f, "Request error: {}", e),
                Self::Utf8Error(e) => write!(f, "UTF-8 conversion error: {}", e),
                Self::Io(e) => write!(f, "IO error: {}", e),
                Self::Server(e) => write!(f, "Server error: {}", e),
                Self::ValidationFailed => write!(f, "File validation failed"),
                _ => {
                    Err(fmt::Error)
                }
            }
        }
    }

    impl std::error::Error for WebError {}

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
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                ResponseBody::Text(text) => write!(f, "{}", text),
                ResponseBody::Bytes(bytes) => {
                    if let Ok(text) = std::str::from_utf8(bytes) {
                        write!(f, "{}", text)
                    } else {
                        write!(f, "Received binary data that is not valid UTF-8")
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
        let response = if way {
            GLOBAL_CLIENT.post(url).json(&body).send()?
        } else {
            GLOBAL_CLIENT.post(url).form(&body).send()?
        };

        let status_code = response.status().as_u16() as i32;
        let content_type = response.headers()
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");

        let res_body = if raw_bytes {
            ResponseBody::Bytes(response.bytes()?.to_vec())
        } else {
            match content_type {
                t if t.contains("text/") || t.contains("json") =>
                    ResponseBody::Text(response.text()?),
                _ => ResponseBody::Bytes(response.bytes()?.to_vec())
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

    #[unsafe(no_mangle)]
    pub extern "C" fn free_c_string(s: *mut c_char) {
        unsafe {
            if !s.is_null() {
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

    // 下载实现优化版
    pub fn download_file<T: AsRef<str>, P: AsRef<Path>>(
        url: T,
        save_path: P,
        requested_threads: usize,
        buffer_pool: &BufferPool,
    ) -> Result<DownloadResult, WebError> {
        let url = url.as_ref();
        let original_path = save_path.as_ref().to_path_buf();
        let temp_path = original_path.with_extension("download");

        // 获取文件信息
        let response = GLOBAL_CLIENT.head(url).send()?;
        let supports_chunked = response.headers()
            .get(ACCEPT_RANGES)
            .map_or(false, |v| v == "bytes");
        let total_size = response.headers()
            .get(CONTENT_LENGTH)
            .and_then(|ct| ct.to_str().ok())
            .and_then(|ct| ct.parse().ok())
            .ok_or(WebError::Server("Missing Content-Length".into()))?;

        // 创建临时文件并设置大小
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&temp_path)?;
        file.set_len(total_size)?;

        // 使用内存映射
        let mut mem_map = unsafe { MmapMut::map_mut(&file) }.map_err(WebError::from)?;

        // 单线程下载（当服务器不支持分块时）
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

        // 多线程下载
        let actual_threads = optimal_thread_count(requested_threads, total_size);
        let chunks = balanced_chunks(total_size, actual_threads);

        // 将 mem_map 按照分块分割成多个可变切片
        let mut slices: Vec<&mut [u8]> = Vec::with_capacity(chunks.len());
        let mut remaining_mem_map = &mut mem_map[..];
        for &(start, end) in &chunks {
            let (left, right) = remaining_mem_map.split_at_mut((end - start) as usize);
            slices.push(left);
            remaining_mem_map = right;
        }

        // 并行处理每个分块
        chunks.par_iter().zip(slices.par_iter_mut()).try_for_each(|(&(start, end), slice)| {
            let client = GLOBAL_CLIENT.clone();
            let mut response = client.get(url)
                .header("Range", format!("bytes={}-{}", start, end))
                .send()?;

            let mut buffer = buffer_pool.get()?; // 从池中获取缓冲区

            loop {
                let read = response.read(&mut buffer)?;
                if read == 0 { break; }
                // 直接操作当前分块的切片
                slice[0..read].copy_from_slice(&buffer[..read]);
            }

            buffer_pool.put(buffer)?; // 将缓冲区归还到池中
            Ok::<(), WebError>(())
        })?;

        // 确保内存映射的内容写入磁盘
        mem_map.flush().map_err(WebError::from)?;

        // 验证和重命名
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
        pub fn get(&self) -> Result<Vec<u8>, WebError> {
            if let Some(buf) = self.pool.pop() {
                Ok(buf)
            } else {
                // 如果缓冲区池为空，创建一个新的缓冲区
                Ok(vec![0; self.buffer_size])
            }
        }

        pub fn put(&self, mut buf: Vec<u8>) -> Result<(), WebError> {
            buf.clear();
            buf.resize(self.buffer_size, 0);
            self.pool.push(buf).map_err(|_| WebError::BufferPoolFull)
        }
    }

    // 辅助函数
    fn optimal_thread_count(requested: usize, total: u64) -> usize {
        let cpu_cores = rayon::current_num_threads();
        let size_based = (total / (1024 * 1024 * 10)) as usize; // 10MB per thread
        requested.clamp(1, cpu_cores.min(size_based).max(1))
    }

    const MIN_CHUNK_SIZE: u64 = 1024 * 1024; // 最小分块大小为 1MB

    fn balanced_chunks(total: u64, threads: usize) -> Vec<(u64, u64)> {
        let mut chunks = Vec::with_capacity(threads);
        let mut remaining = total;
        let mut start = 0;

        for i in 0..threads {
            let chunk_size = if i == threads - 1 {
                remaining // 最后一个分块包含剩余的所有数据
            } else {
                (remaining / (threads - i) as u64).max(MIN_CHUNK_SIZE)
            };

            let end = start + chunk_size - 1;
            chunks.push((start, end));
            start += chunk_size;
            remaining -= chunk_size;
        }

        chunks
    }

    fn validate_file(path: &Path, expected: u64) -> Result<(), WebError> {
        let actual = metadata(path)?.len();
        if actual != expected {
            Err(WebError::ValidationFailed)
        } else {
            Ok(())
        }
    }
}
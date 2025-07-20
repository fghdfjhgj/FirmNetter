pub mod web {
    use crossbeam::queue::ArrayQueue;
    use memmap2::MmapMut;
    use once_cell::sync::Lazy;
    use percent_encoding::percent_decode_str;
    use rayon::iter::IntoParallelRefIterator;
    use rayon::iter::ParallelIterator;
    use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator};
    use reqwest::Url;
    use reqwest::blocking::Client;
    use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_TYPE, HeaderMap};
    use serde::Serialize;
    use std::collections::HashMap;
    use std::ffi::{CStr, CString, c_char};
    use std::fs::{OpenOptions, metadata, rename};
    use std::io::Read;
    use std::os::raw::c_int;
    use std::path::Path;
    use std::ptr;
    use std::time::Duration;

    // 全局HTTP客户端
    static GLOBAL_CLIENT: Lazy<Client> = Lazy::new(|| {
        Client::builder()
            .pool_max_idle_per_host(20)
            .timeout(Duration::from_secs(3000))
            .build()
            .unwrap()
    });

    // 自定义错误类型
    #[derive(Debug)]
    pub enum WebError {
        RequestError(reqwest::Error),
        Utf8Error(std::str::Utf8Error),
        Io(std::io::Error),
        Server(String),
        ValidationFailed,
        BufferPoolEmpty,
        BufferPoolFull,
        InvalidArgument(String),
    }

    // WebError的Display实现
    impl std::fmt::Display for WebError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::RequestError(e) => write!(f, "Request error: {}", e),
                Self::Utf8Error(e) => write!(f, "UTF-8 conversion error: {}", e),
                Self::Io(e) => write!(f, "IO error: {}", e),
                Self::Server(e) => write!(f, "Server error: {}", e),
                Self::ValidationFailed => write!(f, "File validation failed"),
                Self::BufferPoolEmpty => write!(f, "Buffer pool is empty"),
                Self::BufferPoolFull => write!(f, "Buffer pool is full"),
                Self::InvalidArgument(e) => write!(f, "Invalid argument: {}", e),
            }
        }
    }

    // WebError的Error实现
    impl std::error::Error for WebError {}

    // WebError的From转换
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

    // POST请求响应结构体
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

    // ResponseBody的Display实现
    impl std::fmt::Display for ResponseBody {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    // ResPost的构造函数
    impl ResPost {
        pub fn new(status_code: i32, body: ResponseBody) -> ResPost {
            ResPost { status_code, body }
        }
    }

    /// 向指定的 URL 发送 HTTP POST 请求
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

    /// 带自定义头的POST请求
    pub fn web_post_headers<T, B>(
        url: T,
        headers: HeaderMap,
        body: B,
        way: bool,
        raw_bytes: bool,
    ) -> Result<ResPost, WebError>
    where
        T: reqwest::IntoUrl,
        B: Serialize,
    {
        let mut request_builder = if way {
            GLOBAL_CLIENT.post(url).json(&body)
        } else {
            GLOBAL_CLIENT.post(url).form(&body)
        };

        for (name, value) in headers {
            if let Some(name) = name {
                request_builder = request_builder.header(name, value);
            }
        }

        let response = request_builder.send()?;

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
        pub status_code: c_int,
        pub body_type: c_int, // 0: Text, 1: Bytes
        pub body_text: *const c_char,
        pub body_bytes: *const u8,
        pub body_len: usize,
        pub error_msg: *const c_char,
    }

    // C兼容的头信息结构体
    #[repr(C)]
    pub struct CHeaderMap {
        pub keys: *const *const c_char,
        pub values: *const *const c_char,
        pub count: usize,
    }

    // 下载结果结构体
    #[derive(Debug)]
    pub struct DownloadResult {
        pub threads_used: usize,
        pub save_path: String,
        pub file_name: String,
    }

    // C接口的下载结果结构体
    #[repr(C)]
    pub struct CDownloadResult {
        pub threads_used: usize,
        pub save_path: *const c_char,
        pub file_name: *const c_char,
        pub error_msg: *const c_char,
    }

    // 缓冲区池结构体
    pub struct BufferPool {
        pool: ArrayQueue<Vec<u8>>,
        buffer_size: usize,
    }

    // BufferPool的方法实现
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

    /// 计算最优线程数
    fn optimal_thread_count(requested: usize, total: u64) -> usize {
        let cpu_cores = rayon::current_num_threads();
        let size_based = (total / (1024 * 1024 * 10)) as usize;
        requested.clamp(1, cpu_cores.min(size_based).max(1))
    }

    /// 计算均衡分块
    const MIN_CHUNK_SIZE: u64 = 1;
    fn balanced_chunks(total: u64, requested_threads: usize) -> Vec<(u64, u64)> {
        let mut chunks = Vec::new();
        let mut remaining = total;
        let mut start = 0;

        let max_reasonable = (total / MIN_CHUNK_SIZE.max(1)) as usize;
        let actual_threads = requested_threads
            .clamp(1, max_reasonable.max(1))
            .min(rayon::current_num_threads());

        for i in 0..actual_threads {
            let chunk_size = if i == actual_threads - 1 {
                remaining
            } else {
                let avg = remaining / (actual_threads - i) as u64;
                avg.max(MIN_CHUNK_SIZE).min(remaining)
            };

            let end = start + chunk_size.saturating_sub(1);
            chunks.push((start, end));

            start += chunk_size;
            remaining = remaining.saturating_sub(chunk_size);

            if remaining == 0 {
                break;
            }
        }

        chunks
    }

    /// 验证文件大小
    fn validate_file(path: &Path, expected: u64) -> Result<(), WebError> {
        let actual = metadata(path)?.len();
        if actual != expected {
            Err(WebError::ValidationFailed)
        } else {
            Ok(())
        }
    }

    /// 提取文件名
    pub fn extract_filename(url: &str) -> String {
        let parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return "invalid_url.bin".to_string(),
        };

        let path = parsed.path();

        if path.is_empty() || path == "/" {
            return generate_default_name(&parsed);
        }

        let segments: Vec<&str> = path.split('/').collect();

        let filename = segments
            .iter()
            .rev()
            .find(|s| !s.is_empty())
            .map(|s| clean_filename(s))
            .unwrap_or_else(|| generate_default_name(&parsed));

        add_default_extension(filename)
    }

    /// 清理文件名
    fn clean_filename(raw: &str) -> String {
        let decoded = percent_decode_str(raw).decode_utf8().unwrap_or_default();
        decoded.replace(
            |c: char| c.is_control() || c == '/' || c == '\\' || c == ':' || c == '*',
            "_",
        )
    }

    // 生成默认文件名
    fn generate_default_name(url: &Url) -> String {
        let host = url.host_str().unwrap_or("unknown");
        format!("{}.bin", host)
    }

    // 添加默认扩展名
    fn add_default_extension(name: String) -> String {
        if name.contains('.') {
            name
        } else {
            format!("{}.bin", name)
        }
    }

    /// 下载文件的核心逻辑
    pub fn download_file<T: AsRef<str>, P: AsRef<Path>>(
        url: T,
        save_path: P,
        requested_threads: usize,
        mandatory_use: bool,
        buffer_pool: &BufferPool,
    ) -> Result<DownloadResult, WebError> {
        let url = url.as_ref();
        let mut original_path = save_path.as_ref().to_path_buf();

        if original_path.is_dir() {
            let file_name = extract_filename(url);
            original_path = original_path.join(file_name);
        }
        if let Some(parent) = original_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let temp_path = original_path.with_extension("download");

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

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&temp_path)?;
        file.set_len(total_size)?;

        let mut mem_map = unsafe { MmapMut::map_mut(&file)? };

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

        let actual_threads: usize = match mandatory_use {
            true => requested_threads,
            false => optimal_thread_count(requested_threads, total_size),
        };

        let chunks = balanced_chunks(total_size, actual_threads);

        let mut slices: Vec<&mut [u8]> = Vec::with_capacity(chunks.len());
        let mut remaining_mem_map = &mut mem_map[..];
        for &(start, end) in &chunks {
            let chunk_size = (end - start + 1) as usize;
            let (left, right) = remaining_mem_map.split_at_mut(chunk_size);
            slices.push(left);
            remaining_mem_map = right;
        }

        chunks.par_iter().zip(slices.par_iter_mut()).try_for_each(
            |((start, end), slice)| -> Result<(), WebError> {
                download_chunk(&GLOBAL_CLIENT, url, *start, *end, slice, buffer_pool)
            },
        )?;

        mem_map.flush()?;
        validate_file(&temp_path, total_size)?;
        rename(&temp_path, &original_path)?;

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

    // 分块下载函数
    fn download_chunk(
        client: &Client,
        url: &str,
        start: u64,
        end: u64,
        slice: &mut [u8],
        buffer_pool: &BufferPool,
    ) -> Result<(), WebError> {
        const MAX_RETRIES: u8 = 3;
        for attempt in 0..MAX_RETRIES {
            let mut response = client
                .get(url)
                .header("Range", format!("bytes={}-{}", start, end))
                .send()?;

            let mut buffer = buffer_pool.get()?;
            let mut offset = 0;

            loop {
                let read = response.read(&mut buffer)?;
                if read == 0 {
                    break;
                }
                slice[offset..offset + read].copy_from_slice(&buffer[..read]);
                offset += read;
            }

            buffer_pool.put(buffer)?;

            if (end - start + 1) as usize == offset {
                return Ok(());
            }

            if attempt < MAX_RETRIES - 1 {
                eprintln!("Chunk download failed, retrying...");
            }
        }

        Err(WebError::Server(
            "Failed to download chunk after multiple attempts".into(),
        ))
    }

    // 辅助函数：将C字符串转换为Rust字符串
    fn c_str_to_rust_str(c_str: *const c_char) -> Result<&'static str, WebError> {
        unsafe {
            CStr::from_ptr(c_str)
                .to_str()
                .map_err(|e| WebError::Utf8Error(e))
        }
    }

    // 辅助函数：处理C字符串数组到Rust HashMap的转换
    fn convert_c_strings(
        form_data_keys: *const *const c_char,
        form_data_values: *const *const c_char,
        form_data_count: usize,
    ) -> Result<HashMap<String, String>, WebError> {
        let keys = unsafe { std::slice::from_raw_parts(form_data_keys, form_data_count) };
        let values = unsafe { std::slice::from_raw_parts(form_data_values, form_data_count) };

        let mut form_data = HashMap::with_capacity(form_data_count);
        for (k, v) in keys.iter().zip(values.iter()) {
            let key = c_str_to_rust_str(*k)?;
            let value = c_str_to_rust_str(*v)?;
            form_data.insert(key.to_owned(), value.to_owned());
        }
        Ok(form_data)
    }

    // 辅助函数：处理CHeaderMap到HeaderMap的转换
    fn convert_c_headers(headers: *const CHeaderMap) -> Result<HeaderMap, WebError> {
        if headers.is_null() {
            return Ok(HeaderMap::new());
        }

        let header_map = unsafe { &*headers };
        let keys = unsafe { std::slice::from_raw_parts(header_map.keys, header_map.count) };
        let values = unsafe { std::slice::from_raw_parts(header_map.values, header_map.count) };

        let mut result = HeaderMap::new();
        for (k, v) in keys.iter().zip(values.iter()) {
            let key_str = c_str_to_rust_str(*k)?;
            let value_str = c_str_to_rust_str(*v)?;

            let key = match reqwest::header::HeaderName::from_bytes(key_str.as_bytes()) {
                Ok(k) => k,
                Err(_) => continue,
            };

            let value = match reqwest::header::HeaderValue::from_str(value_str) {
                Ok(v) => v,
                Err(_) => continue,
            };

            result.insert(key, value);
        }

        Ok(result)
    }

    // 与C语言交互的POST请求函数
    #[unsafe(no_mangle)]
    pub extern "C" fn c_web_post(
        url: *const c_char,
        form_data_keys: *const *const c_char,
        form_data_values: *const *const c_char,
        form_data_count: usize,
        result: *mut CResPost,
        way: bool,
        raw_bytes: bool,
    ) -> c_int {
        unsafe {
            // 初始化结果结构体
            (*result).status_code = 0;
            (*result).body_type = -1;
            (*result).body_text = ptr::null();
            (*result).body_bytes = ptr::null();
            (*result).body_len = 0;
            (*result).error_msg = ptr::null();

            let url_str = match c_str_to_rust_str(url) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = CString::new(e.to_string()).unwrap_or_default();
                    (*result).error_msg = err_msg.into_raw();
                    return 1;
                }
            };

            let form_data =
                match convert_c_strings(form_data_keys, form_data_values, form_data_count) {
                    Ok(data) => data,
                    Err(e) => {
                        let err_msg = CString::new(e.to_string()).unwrap_or_default();
                        (*result).error_msg = err_msg.into_raw();
                        return 1;
                    }
                };

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
                        }
                        ResponseBody::Bytes(bytes) => {
                            result_ref.body_type = 1;
                            result_ref.body_bytes = bytes.as_ptr();
                            result_ref.body_len = bytes.len();
                        }
                    }
                    0
                }
                Err(e) => {
                    let err_msg = CString::new(e.to_string()).unwrap_or_default();
                    (*result).error_msg = err_msg.into_raw();
                    1
                }
            }
        }
    }

    // 与C语言交互的带头部POST请求函数
    #[unsafe(no_mangle)]
    pub extern "C" fn c_web_post_with_headers(
        url: *const c_char,
        headers: *const CHeaderMap,
        form_data_keys: *const *const c_char,
        form_data_values: *const *const c_char,
        form_data_count: usize,
        result: *mut CResPost,
        way: bool,
        raw_bytes: bool,
    ) -> c_int {
        unsafe {
            // 初始化结果结构体
            (*result).status_code = 0;
            (*result).body_type = -1;
            (*result).body_text = ptr::null();
            (*result).body_bytes = ptr::null();
            (*result).body_len = 0;
            (*result).error_msg = ptr::null();

            let url_str = match c_str_to_rust_str(url) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = CString::new(e.to_string()).unwrap_or_default();
                    (*result).error_msg = err_msg.into_raw();
                    return 1;
                }
            };

            let header_map = match convert_c_headers(headers) {
                Ok(h) => h,
                Err(e) => {
                    let err_msg = CString::new(e.to_string()).unwrap_or_default();
                    (*result).error_msg = err_msg.into_raw();
                    return 1;
                }
            };

            let form_data =
                match convert_c_strings(form_data_keys, form_data_values, form_data_count) {
                    Ok(data) => data,
                    Err(e) => {
                        let err_msg = CString::new(e.to_string()).unwrap_or_default();
                        (*result).error_msg = err_msg.into_raw();
                        return 1;
                    }
                };

            match web_post_headers(url_str, header_map, &form_data, way, raw_bytes) {
                Ok(res_post) => {
                    let result_ref = &mut *result;
                    result_ref.status_code = res_post.status_code;

                    match res_post.body {
                        ResponseBody::Text(text) => {
                            result_ref.body_type = 0;
                            let c_str = CString::new(text)
                                .unwrap_or_else(|_| CString::new("Invalid UTF-8").unwrap());
                            result_ref.body_text = c_str.into_raw();
                        }
                        ResponseBody::Bytes(bytes) => {
                            result_ref.body_type = 1;
                            result_ref.body_bytes = bytes.as_ptr();
                            result_ref.body_len = bytes.len();
                        }
                    }
                    0
                }
                Err(e) => {
                    let err_msg = CString::new(e.to_string()).unwrap_or_default();
                    (*result).error_msg = err_msg.into_raw();
                    1
                }
            }
        }
    }

    // 与C语言交互的下载文件函数
    #[unsafe(no_mangle)]
    pub extern "C" fn c_download_file(
        url: *const c_char,
        save_path: *const c_char,
        requested_threads: usize,
        mandatory_use: bool,
        buffer_pool_size: usize,
        buffer_size: usize,
        result: *mut CDownloadResult,
    ) -> c_int {
        unsafe {
            // 初始化结果结构体
            (*result).threads_used = 0;
            (*result).save_path = ptr::null();
            (*result).file_name = ptr::null();
            (*result).error_msg = ptr::null();

            let url_str = match c_str_to_rust_str(url) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = CString::new(e.to_string()).unwrap_or_default();
                    (*result).error_msg = err_msg.into_raw();
                    return 1;
                }
            };

            let save_path_str = match c_str_to_rust_str(save_path) {
                Ok(s) => s,
                Err(e) => {
                    let err_msg = CString::new(e.to_string()).unwrap_or_default();
                    (*result).error_msg = err_msg.into_raw();
                    return 1;
                }
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
                    result_ref.save_path = c_save_path.into_raw();

                    let c_file_name = CString::new(download_result.file_name)
                        .unwrap_or_else(|_| CString::new("Invalid UTF-8").unwrap());
                    result_ref.file_name = c_file_name.into_raw();

                    0
                }
                Err(e) => {
                    let err_msg = CString::new(e.to_string()).unwrap_or_default();
                    (*result).error_msg = err_msg.into_raw();
                    1
                }
            }
        }
    }

    // 释放CResPost资源
    #[unsafe(no_mangle)]
    pub extern "C" fn free_c_res_post(result: *mut CResPost) {
        if result.is_null() {
            return;
        }
        unsafe {
            // 释放body_text
            if !(*result).body_text.is_null() {
                let _ = CString::from_raw((*result).body_text as *mut c_char);
            }
            // 释放错误信息
            if !(*result).error_msg.is_null() {
                let _ = CString::from_raw((*result).error_msg as *mut c_char);
            }
            // 重置指针
            (*result).body_text = ptr::null();
            (*result).error_msg = ptr::null();
            (*result).body_bytes = ptr::null();
        }
    }

    // 释放CDownloadResult资源
    #[unsafe(no_mangle)]
    pub extern "C" fn free_c_download_result(result: *mut CDownloadResult) {
        if result.is_null() {
            return;
        }
        unsafe {
            // 释放保存路径
            if !(*result).save_path.is_null() {
                let _ = CString::from_raw((*result).save_path as *mut c_char);
            }
            // 释放文件名
            if !(*result).file_name.is_null() {
                let _ = CString::from_raw((*result).file_name as *mut c_char);
            }
            // 释放错误信息
            if !(*result).error_msg.is_null() {
                let _ = CString::from_raw((*result).error_msg as *mut c_char);
            }
            // 重置指针
            (*result).save_path = ptr::null();
            (*result).file_name = ptr::null();
            (*result).error_msg = ptr::null();
        }
    }

    // 导出C兼容的错误码定义
    #[repr(C)]
    pub enum WebErrorCode {
        Success = 0,
        InvalidUrl = 1,
        InvalidPath = 2,
        RequestFailed = 3,
        FileValidationFailed = 4,
        BufferPoolError = 5,
        MemoryAllocationFailed = 6,
        InvalidArgument = 7,
    }

    // 测试函数
    #[test]
    fn test_download_file() {
        let url = "http://api.1wxyun.com/?type=1";
        let mut data = HashMap::new();
        data.insert("Softid", "5T7T5V3G4W1B9Z8Y");
        let res = web_post(url, data, false, false).unwrap();
        println!("status code: {}", res.status_code);
        println!("body: {}", res.body)
    }
}

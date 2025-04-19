pub mod web {
    use reqwest::header::CONTENT_TYPE;
    use serde::Serialize;
    use std::collections::HashMap;
    use std::ffi::{CStr, CString};
    use std::fmt;
    use std::os::raw::c_char;
    use std::ptr;

    /// 自定义错误类型，用于封装可能的请求错误和 UTF-8 转换错误
    #[derive(Debug)]
    pub enum WebError {
        RequestError(reqwest::Error),
        Utf8Error(std::str::Utf8Error),
    }

    impl fmt::Display for WebError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                WebError::RequestError(e) => write!(f, "Request error: {}", e),
                WebError::Utf8Error(e) => write!(f, "UTF-8 conversion error: {}", e),
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

    /// POST 响应结构体，包含状态码和响应体
    #[derive(Debug)]
    pub struct ResPost {
        pub status_code: i32,
        pub body: ResponseBody,
    }

    /// 响应体枚举，可以是文本或字节数组
    #[derive(Debug)]
    pub enum ResponseBody {
        Text(String),
        Bytes(Vec<u8>),
    }

    /// 为 `ResponseBody` 实现 `std::fmt::Display`，以便于打印响应体内容
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
        /// 创建一个新的 `ResPost` 实例
        pub fn new(status_code: i32, body: ResponseBody) -> ResPost {
            ResPost { status_code, body }
        }
    }

    /// 发送 HTTP POST 请求，支持 JSON 和表单数据两种方式
    pub fn web_post<T, B>(
        url: T,
        body: B,
        way: bool,       // true 表示 JSON 格式，false 表示表单格式
        raw_bytes: bool, // 是否获取原始字节
    ) -> Result<ResPost, WebError>
    where
        T: reqwest::IntoUrl,
        B: Serialize,
    {
        let client = reqwest::blocking::Client::new();
        let response = if way {
            client.post(url).json(&body).send()?
        } else {
            client.post(url).form(&body).send()?
        };

        let status_code = response.status().as_u16() as i32;
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");

        // 根据 raw_bytes 参数决定如何处理响应体
        let res_body = if raw_bytes {
            ResponseBody::Bytes(response.bytes()?.to_vec())
        } else if content_type.contains("text/") || content_type.contains("json") {
            ResponseBody::Text(response.text()?)
        } else {
            let bytes = response.bytes()?;
            match std::str::from_utf8(&bytes) {
                Ok(text) => ResponseBody::Text(text.to_string()),
                Err(_) => {
                    ResponseBody::Text("Received binary data that is not valid UTF-8".to_string())
                }
            }
        };

        Ok(ResPost::new(status_code, res_body))
    }

    /// C 结构体用于接收 HTTP POST 请求的结果
    #[repr(C)]
    pub struct CResPost {
        pub status_code: i32,
        pub body_type: i32, // 0 表示文本，1 表示字节数组
        pub body_text: *const c_char,
        pub body_bytes: *const u8,
        pub body_len: usize,
    }

    /// C 接口函数：执行 HTTP POST 请求
    #[unsafe(no_mangle)]
    pub extern "C" fn c_web_post(
        url: *const c_char,
        form_data_keys: *const *const c_char,
        form_data_values: *const *const c_char,
        form_data_count: usize,
        result: *mut CResPost,
        way: bool,
        raw_bytes: bool, // 是否获取原始字节
    ) -> i32 {
        unsafe {
            // 将 C 字符串转换为 Rust 字符串
            let url = match CStr::from_ptr(url).to_str() {
                Ok(u) => u.to_owned(),
                Err(_) => return 1, // 转换失败
            };

            // 构建表单数据的 HashMap
            let mut form_data = HashMap::new();
            for i in 0..form_data_count {
                let key = match CStr::from_ptr(*form_data_keys.offset(i as isize)).to_str() {
                    Ok(k) => k.to_owned(),
                    Err(_) => return 1, // 转换失败
                };
                let value = match CStr::from_ptr(*form_data_values.offset(i as isize)).to_str() {
                    Ok(v) => v.to_owned(),
                    Err(_) => return 1, // 转换失败
                };
                form_data.insert(key, value);
            }

            // 调用核心请求逻辑并处理结果
            match web_post(&url, &form_data, way, raw_bytes) {
                Ok(res_post) => {
                    // 填充响应状态码
                    (*result).status_code = res_post.status_code;

                    // 根据响应体类型填充不同字段
                    match res_post.body {
                        ResponseBody::Text(text) => {
                            (*result).body_type = 0;
                            let c_string = CString::new(text).unwrap();
                            (*result).body_text = c_string.into_raw();
                            (*result).body_bytes = ptr::null();
                            (*result).body_len = 0;
                        }
                        ResponseBody::Bytes(bytes) => {
                            (*result).body_type = 1;
                            (*result).body_text = ptr::null();
                            (*result).body_bytes = bytes.as_ptr();
                            (*result).body_len = bytes.len();
                        }
                    }
                    0 // 成功
                }
                Err(_) => 1, // 失败
            }
        }
    }

    /// 释放 C 字符串，避免内存泄漏
    #[unsafe(no_mangle)]
    pub extern "C" fn free_c_string(s: *mut c_char) {
        unsafe {
            if !s.is_null() {
                let _ = CString::from_raw(s); // 自动释放内存
            }
        }
    }
}

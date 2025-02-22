pub mod web {
    use reqwest::header::CONTENT_TYPE;
    use serde::Serialize;
    use std::collections::HashMap;
    use std::error::Error;
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;
    use std::ptr;

    #[derive(Debug)]
    pub struct ResPost {
        pub status_code: i32,
        pub body: ResponseBody,
    }

    #[derive(Debug)]
    pub enum ResponseBody {
        Text(String),
        Bytes(Vec<u8>),
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

    /// 发起一个POST请求，并根据内容类型处理响应
    ///
    /// # Parameters
    ///
    /// - `url`: 请求的URL，可以是字符串或其他支持转换为URL的类型
    /// - `body`: 请求体，可以是任意可序列化的类型
    /// - `way`: 决定使用JSON还是表单形式发送请求体的布尔值
    ///
    /// # Returns
    ///
    /// 返回一个结果，包含成功时的泛型类型R或错误时的Error类型
    pub fn web_post<T, B, R>(url: T, body: B, way: bool) -> Result<R, Box<dyn Error>>
    where
        T: reqwest::IntoUrl,
        B: Serialize,
        R: From<ResPost>,
    {
        // 创建一个新的HTTP客户端
        let client = reqwest::blocking::Client::new();

        // 根据body的类型自动设置合适的Content-Type
        let response = if way {
            // 如果way为true，以JSON格式发送请求体
            client.post(url).json(&body).send()?
        } else {
            // 如果way为false，以表单形式发送请求体
            client.post(url).form(&body).send()?
        };

        // 获取状态码和内容类型
        let status_code = response.status().as_u16() as i32;
        let content_type = response.headers()
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");

        // 根据内容类型处理响应体
        let res_body = if content_type.contains("text/") || content_type.contains("json") {
            // 如果内容类型为文本或JSON，则处理为文本
            ResponseBody::Text(response.text()?)
        } else {
            // 否则，处理为字节流
            ResponseBody::Bytes(response.bytes()?.to_vec())
        };

        // 使用处理后的响应构建结果类型
        Ok(R::from(ResPost::new(status_code, res_body)))
    }

// 定义一个 C 结构体来接收结果
#[repr(C)]
pub struct CResPost {
    pub status_code: i32,
    pub body_type: i32, // 0 for Text, 1 for Bytes
    pub body_text: *const c_char,
    pub body_bytes: *const u8,
    pub body_len: usize,
}

// 定义一个 C 接口函数
#[no_mangle]
pub extern "C" fn c_web_post(url: *const c_char, form_data_keys: *const *const c_char, form_data_values: *const *const c_char, form_data_count: usize, result: *mut CResPost,way: bool) -> i32 {
    unsafe {
        let url = CStr::from_ptr(url).to_string_lossy().into_owned();
        let mut form_data = HashMap::new();
        for i in 0..form_data_count {
            let key = CStr::from_ptr(*form_data_keys.offset(i as isize)).to_string_lossy().into_owned();
            let value = CStr::from_ptr(*form_data_values.offset(i as isize)).to_string_lossy().into_owned();
            form_data.insert(key, value);
        }

        // 将 HashMap<String, String> 转换为 HashMap<&str, &str>
        let form_data_ref: HashMap<&str, &str> = form_data.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

        match web_post(&url, &form_data_ref,way) {
            Ok(res_post) => {
                (*result).status_code = res_post.status_code;
                match res_post.body {
                    ResponseBody::Text(text) => {
                        (*result).body_type = 0;
                        (*result).body_text = CString::new(text).unwrap().into_raw();
                        (*result).body_bytes = ptr::null();
                        (*result).body_len = 0;
                    },
                    ResponseBody::Bytes(bytes) => {
                        (*result).body_type = 1;
                        (*result).body_text = ptr::null();
                        (*result).body_bytes = bytes.as_ptr();
                        (*result).body_len = bytes.len();
                    },
                }
                0 // 成功
            },
            Err(_) => 1, // 失败
        }
    }

}

// 释放 C 字符串
#[no_mangle]
pub extern "C" fn free_c_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            let _ = CString::from_raw(s);
        }
    }
}

}

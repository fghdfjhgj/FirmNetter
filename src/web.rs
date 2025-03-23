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
        pub fn new(status_code: i32, body: ResponseBody) -> ResPost {
            ResPost { status_code, body }
        }
    }
    /// 将POST响应的主体转换为字符串。
    ///
    /// 该函数用于处理不同类型的响应主体，将其转换为字符串格式以便进一步处理。
    /// 当响应主体为文本时，直接返回；当响应主体为字节时，尝试将其转换为UTF-8字符串。
    /// 如果转换失败，则表示接收到的二进制数据不是有效的UTF-8，返回相应的提示信息。
    ///
    /// # 参数
    ///
    /// * `res_post`: ResPost类型，表示要转换的POST响应。
    ///
    /// # 返回值
    ///
    /// 返回String类型，表示转换后的响应主体。如果接收到的二进制数据不是有效的UTF-8，则返回"Received binary data that is not valid UTF-8"。
    ///
    pub fn convert_res_post_body_to_string(res_post: ResPost) -> String {
        match res_post.body {
            ResponseBody::Text(text) => text,
            ResponseBody::Bytes(bytes) => match std::str::from_utf8(&bytes) {
                Ok(v) => v.to_string(),
                Err(_) => "Received binary data that is not valid UTF-8".to_string(),
            },
        }
    }
    pub fn web_get<T>(url: T) -> Result<ResPost, Box<dyn Error>>
    where
        T: reqwest::IntoUrl,
    {
        let client = reqwest::blocking::Client::new();
        let response = client.get(url).send()?;
        let status_code = response.status().as_u16() as i32;
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|ct| Option::from(ct.to_str().ok().unwrap_or("")))
            .unwrap_or("");
        // 处理响应体
        let res_body = if content_type.contains("text/") || content_type.contains("json") {
            // 如果内容类型为文本或JSON，则将响应体处理为文本
            ResponseBody::Text(response.text()?)
        } else {
            // 否则，将响应体处理为字节流
            ResponseBody::Bytes(response.bytes()?.to_vec())
        };

        // 返回包含状态码和响应体的结果
        Ok(ResPost::new(status_code, res_body))
    }
    /// 向指定URL发送POST请求，并根据响应内容类型处理响应体
    ///
    /// # Parameters
    ///
    /// * `url`: T - 请求的URL，可以是字符串或其他支持转换为URL的类型
    /// * `body`: B - 请求体，可以是任意可序列化为JSON或表单的类型
    /// * `way`: bool - 决定请求体序列化方式的标志，true表示JSON，false表示表单
    ///
    /// # Returns
    ///
    /// * `Result<ResPost, Box<dyn Error>>` - 返回一个结果类型，包含响应状态码和响应体
    ///
    /// # Remarks
    ///
    /// 此函数使用reqwest库发送HTTP请求，并根据响应的内容类型自动处理响应体为文本或字节类型
    pub fn web_post<T, B>(url: T, body: B, way: bool) -> Result<ResPost, Box<dyn Error>>
    where
        T: reqwest::IntoUrl,
        B: Serialize,
    {
        // 创建一个新的HTTP客户端
        let client = reqwest::blocking::Client::new();

        // 根据body的类型自动设置合适的Content-Type
        let response = if way {
            // 如果way为true，将body作为JSON发送
            client.post(url).json(&body).send()?
        } else {
            // 如果way为false，将body作为表单发送
            client.post(url).form(&body).send()?
        };

        // 获取状态码和内容类型
        let status_code = response.status().as_u16() as i32;
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("");

        // 处理响应体
        let res_body = if content_type.contains("text/") || content_type.contains("json") {
            // 如果内容类型为文本或JSON，则将响应体处理为文本
            ResponseBody::Text(response.text()?)
        } else {
            // 否则，将响应体处理为字节流
            ResponseBody::Bytes(response.bytes()?.to_vec())
        };

        // 返回包含状态码和响应体的结果
        Ok(ResPost::new(status_code, res_body))
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

    /// 执行HTTP POST请求的C接口函数
    ///
    /// # 参数
    /// - `url`: *const c_char - C字符串指针，表示请求的URL
    /// - `form_data_keys`: *const *const c_char - 指向C字符串指针数组的指针，表示表单数据的键集合
    /// - `form_data_values`: *const *const c_char - 指向C字符串指针数组的指针，表示表单数据的值集合
    /// - `form_data_count`: usize - 表单数据键值对的数量
    /// - `result`: *mut CResPost - 用于存储请求结果的输出参数指针
    /// - `way`: bool - 控制请求方式的标志位
    ///
    /// # 返回值
    /// - i32: 返回0表示成功，1表示失败
    #[unsafe(no_mangle)]
    pub extern "C" fn c_web_post(
        url: *const c_char,
        form_data_keys: *const *const c_char,
        form_data_values: *const *const c_char,
        form_data_count: usize,
        result: *mut CResPost,
        way: bool,
    ) -> i32 {
        unsafe {
            // 将C字符串转换为Rust字符串
            let url = CStr::from_ptr(url).to_string_lossy().into_owned();

            // 构建表单数据的HashMap
            let mut form_data = HashMap::new();
            for i in 0..form_data_count {
                let key = CStr::from_ptr(*form_data_keys.offset(i as isize))
                    .to_string_lossy()
                    .into_owned();
                let value = CStr::from_ptr(*form_data_values.offset(i as isize))
                    .to_string_lossy()
                    .into_owned();
                form_data.insert(key, value);
            }

            // 将字符串HashMap转换为字符串引用HashMap适配接口
            let form_data_ref: HashMap<&str, &str> = form_data
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();

            // 调用核心请求逻辑并处理结果
            match web_post(&url, &form_data_ref, way) {
                Ok(res_post) => {
                    // 填充响应状态码
                    (*result).status_code = res_post.status_code;

                    // 根据响应体类型填充不同字段
                    match res_post.body {
                        ResponseBody::Text(text) => {
                            (*result).body_type = 0;
                            (*result).body_text = CString::new(text).unwrap().into_raw();
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

    /**
     * 释放由 Rust 分配的 C 风格字符串内存
     *
     * 该函数用于安全释放通过 `CString` 分配到堆内存的字符串，供 C 语言端调用。
     * 当 C 语言代码结束使用该字符串后，必须调用此函数来避免内存泄漏。
     *
     * @param s: *mut c_char - 要释放的 C 字符串指针。如果指针为 NULL 则无操作。
     *                        必须是由 Rust 的 `CString::into_raw()` 生成的指针
     * @return 无返回值
     */
    // 释放 C 字符串
    #[unsafe(no_mangle)]
    pub extern "C" fn free_c_string(s: *mut c_char) {
        unsafe {
            // 重要安全操作：仅处理非空指针
            if !s.is_null() {
                // 通过接管指针所有权实现自动内存回收
                // CString::from_raw 会将指针转换回 CString 并触发析构
                // 使用 let _ 确保析构在当前作用域立即执行
                let _ = CString::from_raw(s);
            }
        }
    }
}

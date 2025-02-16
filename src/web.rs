pub mod web {
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
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

pub fn web_post(url: &str, form_data: &HashMap<&str, &str>) -> Result<ResPost, Box<dyn Error>> {
    let client = Client::new();

    // 发送 POST 请求，并处理可能的错误
    let response = client.post(url)
        .form(form_data)  // 使用表单数据
        .send()?;

    // 获取状态码和内容类型
    let status_code = response.status().as_u16() as i32;
    let content_type = response.headers()
        .get(CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("");

    // 根据内容类型处理响应体
    let body = if content_type.contains("text/") {
        ResponseBody::Text(response.text()?)
    } else {
        // 默认处理为二进制数据
        ResponseBody::Bytes(response.bytes()?.to_vec())
    };

    Ok(ResPost::new(status_code, body))
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
pub extern "C" fn c_web_post(url: *const c_char, form_data_keys: *const *const c_char, form_data_values: *const *const c_char, form_data_count: usize, result: *mut CResPost) -> i32 {
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

        match web_post(&url, &form_data_ref) {
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

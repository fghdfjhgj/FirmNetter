pub mod web {
    use reqwest;
    use std::ffi::CString;
    use std::fs::File;
    use std::io::copy;
    use std::os::raw::c_char;
    use reqwest::blocking::{get, Client};
    use crate::utils::utils::cstring_to_string;
    // 定义一个不会被 Rust 编译器重命名的外部函数，以便与 C 代码互操作
    #[no_mangle]
    extern "C" fn web_get(url: *const c_char) -> *const c_char {
        // 将传入的指针转换为字符串
        let url_str = cstring_to_string(url).expect("Failed to convert C string");
        // 创建一个新的 HTTP 客户端
        let client = Client::new();
        // 发送 GET 请求并处理结果
        match client.get(url_str).send() {
            Ok(res) =>
                // 尝试读取响应内容
                match res.text() {
                    Ok(text) => {
                        // 将响应内容转换为 C 风格的字符串
                        let cstr = CString::new(text).unwrap();
                        cstr.into_raw()
                    }
                    Err(e) => {
                        // 将错误信息转换为 C 风格的字符串
                        let err_str = CString::new(format!("Error reading response: {}", e)).unwrap();
                        err_str.into_raw()
                    }
                },
            Err(e) => {
                // 将错误信息转换为 C 风格的字符串
                let err_str = CString::new(format!("Error sending request: {}", e)).unwrap();
                err_str.into_raw()
            }
        }
    }
    // 定义一个对外的 C 接口，用于执行 HTTP POST 请求
    #[no_mangle]
    extern "C" fn web_post(url: *const c_char,  data: *const c_char) -> *const c_char {
        // 将 URL 的指针和长度转换为字符串
        let url_str = cstring_to_string(url).expect("Failed to convert C string");
        // 将请求数据的指针和长度转换为字符串
        let data_str = cstring_to_string(data).expect("Failed to convert C string");
        // 创建一个新的 HTTP 客户端

        // 发送 POST 请求并处理响应
        let client = match Client::new().post(&url_str).body(data_str).send() {
            Ok(response) => response,
            Err(e) => {
                let err_msg = CString::new(format!("Failed to send request: {}", e)).unwrap();
                return err_msg.into_raw();
            }
        };

        // 获取响应文本
        let response_text = match client.text() {
            Ok(text) => text,
            Err(e) => {
                let err_msg = CString::new(format!("Failed to read response: {}", e)).unwrap();
                return err_msg.into_raw();
            }
        };

        // 将响应文本转换为 C 字符串并返回
        match CString::new(response_text) {
            Ok(cstr) => cstr.into_raw(),
            Err(_) => {
                let err_msg = CString::new("Failed to convert response to CString").unwrap();
                err_msg.into_raw()
            }
        }
    }

    // 定义一个对外的C接口，用于下载文件
    // 该接口接收两个参数：文件的URL和保存的文件名
    // 返回一个C字符串指针，通常用于传递错误信息或成功信息
    #[no_mangle]
    pub extern "C" fn downloader(url: *const c_char, file_name: *const c_char) -> *const c_char {
        // 将 URL 和文件名的指针转换为字符串
        let url = cstring_to_string(url).expect("Failed to convert C string");
        let file_name = cstring_to_string(file_name).expect("Failed to convert C string");

        // 发送 GET 请求
        let response = get(&url);

        match response {
            Ok(mut res) => {
                if res.status().is_success() {
                    // 创建文件
                    let file = File::create(&file_name);
                    match file {
                        Ok(mut file) => {
                            // 将响应内容写入文件
                            if let Err(e) = copy(&mut res, &mut file) {
                                let err_str = CString::new(format!("Error writing to file: {}", e)).unwrap();
                                return err_str.into_raw();
                            }
                            // 返回成功信息
                            let success_str = CString::new("File downloaded successfully").unwrap();
                            success_str.into_raw()
                        }
                        Err(e) => {
                            let err_str = CString::new(format!("Error creating file: {}", e)).unwrap();
                            err_str.into_raw()
                        }
                    }
                } else {
                    let err_str = CString::new(format!("Failed to download file, status code: {}", res.status())).unwrap();
                    err_str.into_raw()
                }
            }
            Err(e) => {
                let err_str = CString::new(format!("Error sending request: {}", e)).unwrap();
                err_str.into_raw()
            }
        }
    }
}



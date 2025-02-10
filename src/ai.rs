pub mod ai{
    use crate::utils::utils::cstring_to_string;
    use crate::utils::utils::str_to_cstr;
    use reqwest::blocking::Client;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::ffi::{c_char, c_float, c_int};

    #[derive(Serialize, Debug)]
    struct ChatMessage {
        role: String,
        content: String,
    }

    #[derive(Serialize)]
    struct ChatRequest {
        model: String,
        messages: Vec<ChatMessage>,
    }

    #[derive(Deserialize, Debug)]
    struct ChatResponse {
        choices: Vec<ChatChoice>,
    }

    #[derive(Deserialize, Debug)]
    struct ChatChoice {
        message: ChatMessage,
    }
    /// 发送请求以获取AI响应的外部接口函数。
///
/// # 参数
/// - `url`: 指向C字符串的指针，表示API的URL。
/// - `api_ket`: 指向C字符串的指针，表示API密钥。
/// - `modle`: 指向C字符串的指针，表示使用的模型名称。
/// - `role`: 指向C字符串的指针，表示消息发送者的角色。
/// - `content`: 指向C字符串的指针，表示消息内容。
/// - `temperature`: 浮点数，表示生成文本的随机性程度。
/// - `max_tokens`: 整数，表示生成文本的最大长度。
/// - `top_p`: 浮点数，表示用于采样的概率阈值。
/// - `n`: 整数，表示生成的回复数量。
/// - `stop`: 指向C字符串的指针，表示停止生成的序列。

///
/// # 返回值
/// - 返回指向C字符串的指针，表示API的响应结果或错误信息。
#[no_mangle]
pub extern "C" fn get_ai_no_stream(url: *const c_char,
                         api_ket: *const c_char,
                         modle: *const c_char,
                         role: *const c_char,
                         content: *const c_char,
                         temperature: c_float,
                         max_tokens: c_int,
                         top_p: c_float,
                         n: c_int,
                         stop: *const c_char
) -> *const c_char {
    // 将C字符串转换为Rust字符串，并处理可能的转换失败
    let url_str = cstring_to_string(url).expect("Failed to convert C string");
    let api_key = cstring_to_string(api_ket).expect("Failed to convert C string");
    let modle_str = cstring_to_string(modle).expect("Failed to convert C string");
    let role_str = cstring_to_string(role).expect("Failed to convert C string");
    let content_str = cstring_to_string(content).expect("Failed to convert C string");
    let stop_str = cstring_to_string(stop).expect("Failed to convert C string");

    // 构建JSON请求体
    let json_data = json!({
        "model": modle_str,
        "messages": [
            {"role": role_str, "content": content_str}
        ],
        "temperature": temperature,
        "max_tokens": max_tokens,
        "top_p": top_p,
        "n": n,
        "stop": stop_str,
        "stream": false
    });

    // 创建HTTP客户端并发送POST请求
    let client = Client::new();
    let res = client
        .post(url_str)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json_data)
        .send()
        .expect("Failed to send request");

    // 处理响应结果
    match res {
        Ok(_res) => {
            str_to_cstr(res.text().expect("Failed to get response text"))
        }
        Err(_err) => {
            str_to_cstr("Failed to send request".parse().unwrap())
        }
    }
}
    // 获取AI流式响应文本
    //
    // 该函数通过指定的URL和API密钥向AI模型发送请求，并以流式方式接收响应。
    // 它允许用户指定模型、角色、内容以及生成文本的 various 参数，如温度、最大令牌数等。
    pub async fn get_ai_stream(
        url: &str,
        api_key: &str,
        model: &str,
        role: &str,
        content: &str,
        temperature: f32,
        max_tokens: i32,
        top_p: f32,
        n: i32,
        stop: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // 构建JSON请求体
        let json_data = json!({
        "model": model,
        "messages": [
            {"role": role, "content": content}
        ],
        "temperature": temperature,
        "max_tokens": max_tokens,
        "top_p": top_p,
        "n": n,
        "stop": stop,
        "stream": true // 启用流式传输
    });

        // 创建HTTP客户端
        let client = Client::new();

        // 发送POST请求
        let res = client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&json_data)
            .send()
            .await?;

        // 检查响应状态
        if !res.status().is_success() {
            return Err(format!("Request failed: {}", res.status()).into());
        }

        // 以流式方式读取响应体
        let mut stream = res.bytes_stream();
        let mut result = String::new();

        // 逐步处理流式数据
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            // 将字节数据转换为字符串并追加到结果中
            let chunk_str = String::from_utf8_lossy(&chunk);
            result.push_str(&chunk_str);
        }

        Ok(result)
    }
#[no_mangle]
pub extern "C" fn C_get_ai_stream(url: *const c_char,
                                  api_ket: *const c_char,
                                  modle: *const c_char,
                                  role: *const c_char,
                                  content: *const c_char,
                                  temperature: c_float,
                                  max_tokens: c_int,
                                  top_p: c_float,
                                  n: c_int,
                                  stop: *const c_char)-> *const c_char {
    let url_str = cstring_to_string(url).expect("Failed to convert C string");
    let api_key = cstring_to_string(api_ket).expect("Failed to convert C string");
    let modle_str = cstring_to_string(modle).expect("Failed to convert C string");
    let role_str = cstring_to_string(role).expect("Failed to convert C string");
    let content_str = cstring_to_string(content).expect("Failed to convert C string");
    let stop_str = cstring_to_string(stop).expect("Failed to convert C string");
    let result = get_ai_stream(
        &url_str,
        &api_key,
        &modle_str,
        &role_str,
        &content_str,
        temperature,
        max_tokens,
        top_p,
        n,
        &stop_str,
    );
    match result {
        Ok(result) => {
            str_to_cstr(result)
        }
        Err(_err) => {
            str_to_cstr("Failed to send request".parse().unwrap())
        }
    }

}
}
pub mod web {
    use reqwest::blocking::Client;
    use reqwest::header::CONTENT_TYPE;
    use std::collections::HashMap;
    use std::error::Error;
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
}

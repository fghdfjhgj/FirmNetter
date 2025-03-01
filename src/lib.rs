pub mod utils;
pub use utils::utils as other_utils;
pub mod ai;
pub mod flash_phone;
pub mod web;
pub use ai::ai as other_ai;
mod kernel;
pub mod sql;
pub use flash_phone::flash_phone as other_flash_phone;
pub use kernel::kernel as other_kernel;
pub use sql::sql as other_sql;
pub use web::web as other_web;
#[cfg(test)]
mod tests {
    use crate::other_web::{ResponseBody, web_post};
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        let mut form_data = HashMap::new();
        form_data.insert("Softid", "0H9G1H8Q5O9G0H2Z");
        let a = web_post("http://api.1wxyun.com/?type=1", form_data, false);

        let response_string = match a.unwrap().body {
            ResponseBody::Text(text) => text,
            ResponseBody::Bytes(bytes) => match std::str::from_utf8(&bytes) {
                Ok(v) => v.to_string(),
                Err(_) => String::from("Received binary data that is not valid UTF-8"),
            },
        };
        println!("{}", response_string);
    }
}

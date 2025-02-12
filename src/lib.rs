pub mod utils;
pub use utils::utils as other_utils;
pub mod web;
pub mod flash_phone;
pub mod ai;
pub use ai::ai as other_ai ;
pub mod sql;
mod kernel;
pub use kernel::kernel as other_kernel ;
pub use flash_phone::flash_phone as other_flash_phone ;
pub use sql::sql as other_sql ;
pub use web::web as other_web ;
#[cfg(test)]
mod tests {

    use crate::utils::utils::*;
    #[test]
    fn it_works() {
        set_console_output_cp_to_utf8();
        let c=str_to_cstr("adb devices".parse().unwrap());
        let result = exec(c).stderr;
        println!("{}",cstring_to_string(result).expect("error"));
    }
}
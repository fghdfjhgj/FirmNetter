pub mod kernel{
    use crate::utils::utils::{exec, str_to_cstr};


#[no_mangle]
/// 解包图像文件的 C 兼容接口函数。
///
/// 此函数用于调用外部 `magisk.exe` 工具来解包图像文件，并根据传入的标志参数决定是否执行特定操作或保留头部信息。
///
/// # 参数
///
/// * `_n` - 布尔值，指示是否需要执行特定的解包操作。如果为 `true`，则传递 `-n` 参数给 `magisk.exe`。
/// * `_h` - 布尔值，指示是否需要保留头部信息。如果为 `true`，则传递 `-h` 参数给 `magisk.exe`。
///
/// # 返回值
///
/// * `bool` - 返回解包操作是否成功的布尔值。成功返回 `true`，失败返回 `false`。
///
/// # 重要代码块说明
///
/// - 根据 `_n` 和 `_h` 参数构建命令行参数字符串。
/// - 调用 `exec` 函数执行 `magisk.exe unpack` 命令，并将构建的参数传递给该命令。
/// - 最终返回命令执行的成功状态。
pub extern "C" fn unpack_img(_n: bool, _h: bool) -> bool {
    // 根据 _n 标志决定是否添加 "-n" 参数
    let a = if _n { "-n" } else { "" };

    // 根据 _h 标志决定是否添加 "-h" 参数
    let b = if _h { "-h" } else { "" };

    // 构建并执行 magisk.exe unpack 命令，返回命令执行的成功状态
    exec(str_to_cstr(format!("magisk.exe unpack {} {}", a, b))).success
    }
}


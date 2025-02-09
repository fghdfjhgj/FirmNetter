pub mod flash_phone {
    use crate::utils;
    use std::ffi::{c_char, CString};
    use std::ptr;
    use serde::{Deserialize, Serialize};
    use utils::utils::*;

    /// 表示没有root权限的Android手机信息。
    ///
    /// 该结构体包含指向表示各种Android系统信息的字符字符串指针。
    /// 使用 `#[repr(C)]` 确保结构体在内存中的布局与C语言约定兼容，
    /// 便于Rust与C代码之间的互操作，特别是在访问Android的原生API时非常有用。
    ///
    #[repr(C)]
    #[derive(Serialize, Deserialize)]
    pub struct NoRootPhoneData {
        /// 指向表示Android内核版本的字符字符串的指针。
        kernel_version: *const c_char,
        /// 指向表示Android版本的字符字符串的指针。
        android_version: *const c_char,
        /// 指向表示Android SDK版本的字符字符串的指针。
        sdk_version: *const c_char,
        /// 指向表示产品制造商的字符字符串的指针。
        ro_product_manufacturer: *const c_char,
        /// 指向表示产品CPU ABI（应用程序二进制接口）的字符字符串的指针。
        ro_cpu_abi: *const c_char,
        /// 指向表示产品品牌的字符字符串的指针。
        ro_product_brand: *const c_char,
        /// 指向表示产品型号的字符字符串的指针。
        ro_product_model: *const c_char,
        /// 指向表示产品设备名称的字符字符串的指针。
        ro_product_device: *const c_char,
        /// 指向表示硬件名称的字符字符串的指针。
        ro_hardware: *const c_char,
        /// 指向表示构建描述的字符字符串的指针。
        ro_build_description: *const c_char,
        /// 指向表示安全补丁版本的字符字符串的指针。
        ro_build_version_security_patch: *const c_char,
        /// 指向表示构建ID的字符字符串的指针。
        ro_build_id: *const c_char,
        /// 指向表示引导加载程序版本的字符字符串的指针。
        ro_bootloader: *const c_char,
        /// 指向表示调制解调器软件版本的字符字符串的指针。
        ro_modem_software_version: *const c_char,
        /// 指向表示内核QEMU标志的字符字符串的指针。
        ro_kernel_qemu: *const c_char,
    }

    impl NoRootPhoneData {
        /// 创建一个新的NoRootPhoneData实例
        ///
        /// NoRootPhoneData结构体用于存储Android设备的相关信息，这些信息通常用于在不需要root权限的情况下获取设备的硬件和软件配置
        /// 此函数初始化一个NoRootPhoneData实例，并将所有字段设置为null，表示未初始化或不可用的数据
        ///
        /// 返回值:
        /// 返回一个NoRootPhoneData实例，其中所有字段都被初始化为null

        fn new() -> NoRootPhoneData {
            NoRootPhoneData {
                kernel_version: ptr::null(),
                android_version: ptr::null(),
                sdk_version: ptr::null(),
                ro_product_manufacturer: ptr::null(),
                ro_cpu_abi: ptr::null(),
                ro_product_brand: ptr::null(),
                ro_product_model: ptr::null(),
                ro_product_device: ptr::null(),
                ro_hardware: ptr::null(),
                ro_build_description: ptr::null(),
                ro_build_version_security_patch: ptr::null(),
                ro_build_id: ptr::null(),
                ro_bootloader: ptr::null(),
                ro_modem_software_version: ptr::null(),
                ro_kernel_qemu: ptr::null(),
            }
        }
    }

    /// 释放 NoRootPhoneData 结构体中的资源
    ///
    /// 在某些情况下，我们可能需要直接与 C 语言代码交互，或手动管理内存，这时就需要使用 `extern "C"` 函数
    /// 并通过手动释放内存来避免内存泄漏。本函数旨在释放 NoRootPhoneData 结构体中指向的字符串资源
    /// 这些字符串资源在 C 语言环境中创建，使用完毕后需要手动释放
    ///
    /// # 参数
    ///
    /// * `data` - 一个可变引用，指向 NoRootPhoneData 结构体。这个结构体包含了多个指向 C 语言字符串的指针
    #[no_mangle]
    pub extern "C" fn free_no_root_phone_data(data: &mut NoRootPhoneData) {
        // 在 Rust 中，使用裸指针和直接内存管理相关的操作被认为是不安全的
        // 因此，我们需要在一个 unsafe 块中执行这些操作
        unsafe {
            // 检查每个字符串指针是否为 null，如果不为 null，则使用 CString::from_raw 将其转换并释放
            // 注意：转换后CString的所有权归 Rust 所有，Rust 会在其作用域结束时自动释放内存
            if !data.kernel_version.is_null() { let _ = CString::from_raw(data.kernel_version as *mut c_char); }
            if !data.android_version.is_null() { let _ = CString::from_raw(data.android_version as *mut c_char); }
            if !data.sdk_version.is_null() { let _ = CString::from_raw(data.sdk_version as *mut c_char); }
            if !data.ro_product_manufacturer.is_null() { let _ = CString::from_raw(data.ro_product_manufacturer as *mut c_char); }
            if !data.ro_cpu_abi.is_null() { let _ = CString::from_raw(data.ro_cpu_abi as *mut c_char); }
            if !data.ro_product_brand.is_null() { let _ = CString::from_raw(data.ro_product_brand as *mut c_char); }
            if !data.ro_product_model.is_null() { let _ = CString::from_raw(data.ro_product_model as *mut c_char); }
            if !data.ro_product_device.is_null() { let _ = CString::from_raw(data.ro_product_device as *mut c_char); }
            if !data.ro_hardware.is_null() { let _ = CString::from_raw(data.ro_hardware as *mut c_char); }
            if !data.ro_build_description.is_null() { let _ = CString::from_raw(data.ro_build_description as *mut c_char); }
            if !data.ro_build_version_security_patch.is_null() { let _ = CString::from_raw(data.ro_build_version_security_patch as *mut c_char); }
            if !data.ro_build_id.is_null() { let _ = CString::from_raw(data.ro_build_id as *mut c_char); }
            if !data.ro_bootloader.is_null() { let _ = CString::from_raw(data.ro_bootloader as *mut c_char); }
            if !data.ro_modem_software_version.is_null() { let _ = CString::from_raw(data.ro_modem_software_version as *mut c_char); }
            if !data.ro_kernel_qemu.is_null() { let _ = CString::from_raw(data.ro_kernel_qemu as *mut c_char); }
        }

        // 将指针重置为 null，以防再次使用时出现未定义行为
        // 这是一个好习惯，特别是在手动管理内存时，可以防止悬挂指针的出现
        *data = NoRootPhoneData::new();
    }

    #[repr(C)]
    #[derive(Serialize, Deserialize)]
    pub struct RootPhoneData {
        root_ro_serialno: *const c_char,
    }

    /// 获取非root手机数据
    ///
    /// 该函数通过ADB命令从指定的设备获取各种系统信息，包括内核版本、Android版本、SDK版本等，
    /// 并将这些信息封装到一个NoRootPhoneData结构体中返回。
    ///
    /// # 参数
    /// * `id` - 设备ID的C字符串指针，用于指定要获取信息的设备。
    ///
    /// # 返回值
    /// 返回一个指向NoRootPhoneData结构体的指针，该结构体包含了从设备获取的所有系统信息。
    ///
    /// # 安全性
    /// 调用者需要确保传入的`id`参数是有效的，并且在使用返回的指针后正确地管理内存，
    /// 以避免内存泄漏或未定义行为。
    #[no_mangle]
    pub extern "C" fn get_no_root_phone_data(id: *const c_char) -> *mut NoRootPhoneData {
        let id_str = cstring_to_string(id).expect("error");

        let properties = vec![
            ("kernel_version", format!("adb -s {} shell uname -r", id_str)),
            ("android_version", format!("adb -s {} shell getprop ro.build.version.release", id_str)),
            ("sdk_version", format!("adb -s {} shell getprop ro.build.version.sdk", id_str)),
            ("ro_product_manufacturer", format!("adb -s {} shell getprop ro.product.manufacturer", id_str)),
            ("ro_cpu_abi", format!("adb -s {} shell getprop ro.product.cpu.abi", id_str)),
            ("ro_product_brand", format!("adb -s {} shell getprop ro.product.brand", id_str)),
            ("ro_product_model", format!("adb -s {} shell getprop ro.product.model", id_str)),
            ("ro_product_device", format!("adb -s {} shell getprop ro.product.device", id_str)),
            ("ro_hardware", format!("adb -s {} shell getprop ro.hardware", id_str)),
            ("ro_build_description", format!("adb -s {} shell getprop ro.build.description", id_str)),
            ("ro_build_version_security_patch", format!("adb -s {} shell getprop ro.build.version.security_patch", id_str)),
            ("ro_build_id", format!("adb -s {} shell getprop ro.build.id", id_str)),
            ("ro_bootloader", format!("adb -s {} shell getprop ro.bootloader", id_str)),
            ("ro_modem_software_version", format!("adb -s {} shell getprop ro.modem.software.version", id_str)),
            ("ro_kernel_qemu", format!("adb -s {} shell getprop ro.kernel.qemu", id_str)),
        ];

        let mut no_root_phone_data = NoRootPhoneData::new();

        for (field, command) in properties {
            let cstr = CString::new(cstring_to_string(exec(str_to_cstr(command)).stdout).expect("error").into_bytes()).expect("CString::new failed");
            match field {
                "kernel_version" => no_root_phone_data.kernel_version = cstr.into_raw(),
                "android_version" => no_root_phone_data.android_version = cstr.into_raw(),
                "sdk_version" => no_root_phone_data.sdk_version = cstr.into_raw(),
                "ro_product_manufacturer" => no_root_phone_data.ro_product_manufacturer = cstr.into_raw(),
                "ro_cpu_abi" => no_root_phone_data.ro_cpu_abi = cstr.into_raw(),
                "ro_product_brand" => no_root_phone_data.ro_product_brand = cstr.into_raw(),
                "ro_product_model" => no_root_phone_data.ro_product_model = cstr.into_raw(),
                "ro_product_device" => no_root_phone_data.ro_product_device = cstr.into_raw(),
                "ro_hardware" => no_root_phone_data.ro_hardware = cstr.into_raw(),
                "ro_build_description" => no_root_phone_data.ro_build_description = cstr.into_raw(),
                "ro_build_version_security_patch" => no_root_phone_data.ro_build_version_security_patch = cstr.into_raw(),
                "ro_build_id" => no_root_phone_data.ro_build_id = cstr.into_raw(),
                "ro_bootloader" => no_root_phone_data.ro_bootloader = cstr.into_raw(),
                "ro_modem_software_version" => no_root_phone_data.ro_modem_software_version = cstr.into_raw(),
                "ro_kernel_qemu" => no_root_phone_data.ro_kernel_qemu = cstr.into_raw(),
                _ => {}
            }
        }

        Box::into_raw(Box::new(no_root_phone_data))
    }

    /// 获取指定设备的根手机数据
    ///
    /// 本函数通过ADB命令获取指定设备的属性信息，并将其封装为RootPhoneData类型返回
    /// 主要用于需要从设备序列号获取设备详细信息的场景
    ///
    /// # 参数
    /// * `id` (*const c_char): 设备的序列号，用于ADB命令中指定设备
    ///
    /// # 返回值
    /// * `*mut RootPhoneData`: 返回一个指向RootPhoneData类型的可变指针，包含设备的属性信息
    ///
    /// # 安全性
    /// 调用此函数时需要注意内存管理，避免内存泄漏和无效指针访问
    /// 调用者需要在使用完返回的数据后适当释放内存
    #[no_mangle]
    pub extern "C" fn get_root_phone_data(id: *const c_char) -> *mut RootPhoneData {
        let id_str = cstring_to_string(id).expect("error");
        let res = exec(str_to_cstr(format!("adb -s {} shell getprop", id_str))).stdout;

        let root_phone_data = RootPhoneData {
            root_ro_serialno: res,
        };

        Box::into_raw(Box::new(root_phone_data))
    }

    /// 执行Fastboot命令

      pub fn execute_fastboot_command(id: *const c_char, command:*const c_char,parameter:*const c_char) -> *const c_char {
        let res = exec(str_to_cstr(format!("fastboot -s {} {} {}", cstring_to_string(id).expect("error"), cstring_to_string(command).expect("error"), cstring_to_string(parameter).expect("error"))));
        let result = if res.success {
            res.stdout
        } else {
            res.stderr
        };
        free_command_result(res);
        result
    }

    #[no_mangle]
    pub extern "C" fn flash_boot_a(id: *const c_char, path: *const c_char) -> *const c_char {
        execute_fastboot_command(id, str_to_cstr("flash boot_a".parse().unwrap()),path)
    }

    #[no_mangle]
    pub extern "C" fn flash_boot_b(id: *const c_char, path: *const c_char) -> *const c_char {
        execute_fastboot_command(id, str_to_cstr("flash boot_b".parse().unwrap()), path)
    }

    #[no_mangle]
    pub extern "C" fn flash_boot(id: *const c_char, path: *const c_char) -> *const c_char {
        execute_fastboot_command(id, str_to_cstr("flash boot".parse().unwrap()), path)
    }

    #[no_mangle]
    pub extern "C" fn flash_recovery(id: *const c_char, path: *const c_char) -> *const c_char {
        execute_fastboot_command(id, str_to_cstr("flash recovery".parse().unwrap()), path)
    }

    #[no_mangle]
    pub extern "C" fn flash_init_boot(id: *const c_char, path: *const c_char) -> *const c_char {
        execute_fastboot_command(id, str_to_cstr("init_boot".parse().unwrap()), path)
    }

    #[no_mangle]
    pub extern "C" fn install_app(id: *const c_char, path: *const c_char, debug: bool, repeat: bool) -> *const c_char {
        let debug_new = if debug { "-t" } else { "" };
        let repeat_new = if repeat { "-r" } else { "" };
        let id_str = cstring_to_string(id).expect("error");
        let path_str = cstring_to_string(path).expect("error");
        let res = exec(str_to_cstr(format!("adb -s {} {} {} install {}", id_str, debug_new, repeat_new, path_str)));
        if res.success {
            str_to_cstr(cstring_to_string(res.stdout).expect("REASON"))
        } else {
            str_to_cstr(cstring_to_string(res.stderr).expect("Failed to convert stdout"))
        }
    }
     pub fn execute_adb_command(id: *const c_char, command: *const c_char,parameter:*const c_char) -> *const c_char {
        let res = exec(str_to_cstr(format!("adb -s  {}  {}  {}", cstring_to_string(id).expect("error"), cstring_to_string(command).expect("error"), cstring_to_string(parameter).expect("error"))));
        let result = if res.success {
            res.stdout
        } else {
            res.stderr
        };
        free_command_result(res);
        result
    }

    #[no_mangle]
    pub extern "C" fn adb_devices_phone() -> *const c_char {
        exec(str_to_cstr("adb devices".parse().unwrap())).stdout

    }

    #[no_mangle]
    pub extern "C" fn fastboot_devices_phone() -> *const c_char {
        exec(str_to_cstr("fastboot devices".parse().unwrap())).stdout
    }

    #[no_mangle]
    pub extern "C" fn get_kernel_version(id: *const c_char) -> *const c_char {
        execute_adb_command(id, str_to_cstr("uname".parse().unwrap()),str_to_cstr("-r".parse().unwrap()))
    }

    /// 根据给定的设备ID检查当前的启动槽
    ///
    /// 此函数通过调用外部命令来获取当前的启动槽信息
    /// 它使用fastboot命令行工具与设备通信，指定设备ID，并请求当前槽的信息
    ///
    /// # 参数
    ///
    /// * `id`: 指向C风格字符串的指针，表示设备ID
    ///
    /// # 返回
    ///
    /// 返回指向C风格字符串的指针，表示当前的启动槽信息或错误信息
    #[no_mangle]
    pub extern "C" fn check_current_slot(id: *const c_char) -> *const c_char {
        // 将C风格字符串转换为Rust字符串
        let id_str = cstring_to_string(id).expect("error");

        // 构造并执行命令，获取当前启动槽的信息
        let res = exec(str_to_cstr(format!("fastboot -s {} getvar current-slot", id_str)));

        // 根据执行结果返回相应的信息或错误信息
        if res.success {
            str_to_cstr(cstring_to_string(res.stdout).expect("REASON"))
        } else {
            str_to_cstr(cstring_to_string(res.stderr).expect("REASON"))
        }
    }
    /// 执行手机命令
    ///
    /// 该函数通过ADB命令在指定的设备上执行shell命令它接受设备ID和要执行的命令作为参数，
    /// 并返回命令执行的结果这个函数被标记为`no_mangle`，以便它的名称不会被Rust的名称修饰
    /// 机制修改，并使用C调用约定，使其可以被C代码调用
    ///
    /// # 参数
    ///
    /// * `id` - 设备ID的C字符串指针
    /// * `command` - 要执行的命令的C字符串指针
    ///
    /// # 返回值
    ///
    /// * 执行成功或失败的C字符串指针
    #[no_mangle]
    pub extern "C" fn phone_exec(id: *const c_char, command: *const c_char) -> *const c_char {
        // 将C字符串转换为Rust字符串
        let id_str = cstring_to_string(id).expect("error");
        let command_str = cstring_to_string(command).expect("error");

        // 构造并执行ADB命令
        let res = exec(str_to_cstr(format!("adb -s {} shell {}", id_str, command_str)));

        // 根据命令执行结果返回相应的C字符串
        if res.success {
            str_to_cstr(cstring_to_string(res.stdout).expect("REASON"))
        } else {
            str_to_cstr(cstring_to_string(res.stderr).expect("REASON"))
        }
    }
    /// 启动手机的不同模式
    ///
    /// 此函数通过ADB命令启动手机进入不同的模式，如重启、恢复模式、bootloader模式等。
    /// 它根据输入的`modle`参数决定启动哪种模式，并通过`id`参数指定的设备执行相应命令。
    ///
    /// # 参数
    /// - `id`: 设备标识符的C风格字符串指针通过此参数指定要操作的设备。
    /// - `modle`: 模式代码一个整数，表示要启动的模式类型。
    ///
    /// # 返回值
    /// 返回一个C风格字符串指针，表示执行结果或错误信息。
    #[no_mangle]
    pub extern "C" fn adb_phone_start(id: *const c_char, modle: i32) -> *const c_char {
        // 将C风格字符串转换为Rust字符串
        let id_str = cstring_to_string(id).expect("error");
        let modles;

        // 根据modle参数选择不同的启动模式
        match modle {
            1 => {
                modles = "reboot";
            }
            2 => {
                modles = "reboot recovery";
            }
            3 => {
                modles = "reboot bootloader";
            }
            4 => {
                modles = "reboot fastboot";
            }
            5 => {
                modles = "reboot edl";
            }
            _ => {
                // 如果modle参数不匹配任何已知模式，返回错误信息
                return str_to_cstr("error".parse().unwrap());
            }
        }

        // 执行ADB命令
        let res = exec(str_to_cstr(format!("adb -s {} shell {}", id_str, modles)));
        // 根据命令执行结果返回相应的C风格字符串
        if res.success {
            str_to_cstr(cstring_to_string(res.stdout).expect("REASON"))
        } else {
            str_to_cstr(cstring_to_string(res.stderr).expect("REASON"))
        }
    }
    #[no_mangle]
    pub extern "C" fn fastboot_phone_start(id: *const c_char, modle: i32) -> *const c_char {
        // 将C风格字符串转换为Rust字符串
        let id_str = cstring_to_string(id).expect("error");
        let  modles;

        // 根据modle参数选择不同的启动模式
        match modle {
            1 => {
                modles = "reboot";
            }
            2 => {
                modles = "reboot recovery";
            }
            3 => {
                modles = "reboot bootloader";
            }
            4 => {
                modles = "reboot fastboot";
            }
            5 => {
                modles = "reboot edl";
            }
            _ => {
                // 如果modle参数不匹配任何已知模式，返回错误信息
                return str_to_cstr("error".parse().unwrap());
            }
        }

        // 执行fastboot命令
        let res = exec(str_to_cstr(format!("fastboot -s {} shell {}", id_str, modles)));
        // 根据命令执行结果返回相应的C风格字符串
        if res.success {
            str_to_cstr(cstring_to_string(res.stdout).expect("REASON"))
        } else {
            str_to_cstr(cstring_to_string(res.stderr).expect("REASON"))
        }
    }
   

}

pub mod flash_phone {
    use crate::other_utils::{cstring_to_string, free_command_result};
    use crate::utils::utils;
    use std::ffi::{c_char, CString};
    use std::ptr;

   /// 表示没有root权限的Android手机信息。
///
/// 该结构体包含指向表示各种Android系统信息的字符字符串指针。
/// 使用 `#[repr(C)]` 确保结构体在内存中的布局与C语言约定兼容，
/// 便于Rust与C代码之间的互操作，特别是在访问Android的原生API时非常有用。
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

    pub extern "C" fn free_no_root_phone_data(data: &mut NoRootPhoneData) {
        unsafe {
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
        *data = NoRootPhoneData::new();
    }
    #[repr(C)]
    pub struct RootPhoneData {
        root_ro_serialno: *const c_char,
    }
    #[no_mangle]
    pub extern "C" fn get_no_root_phone_data(id:*const c_char) -> *mut NoRootPhoneData {
        let kernel_version_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell uname -r ", cstring_to_string(id).expect("error")))).stdout;
        let andord_version_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.build.version.release", cstring_to_string(id).expect("error")))).stdout;
        let sdk_version_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.build.version.sdk", cstring_to_string(id).expect("error")))).stdout;
        let ro_product_manufacturer_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.product.manufacturer", cstring_to_string(id).expect("error")))).stdout;
        let ro_cpu_abi_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.product.cpu.abi", cstring_to_string(id).expect("error")))).stdout;
        let ro_product_brand_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.product.brand", cstring_to_string(id).expect("error")))).stdout;
        let ro_product_model_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.product.model", cstring_to_string(id).expect("error")))).stdout;
        let ro_product_device_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.product.device", cstring_to_string(id).expect("error")))).stdout;
        let ro_hardware_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.hardware", cstring_to_string(id).expect("error")))).stdout;
        let ro_build_description_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.build.description", cstring_to_string(id).expect("error")))).stdout;
        let ro_build_version_security_patch_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.build.version.security_patch", cstring_to_string(id).expect("error")))).stdout;
        let ro_build_id_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.build.id", cstring_to_string(id).expect("error")))).stdout;
        let ro_bootloader_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.bootloader", cstring_to_string(id).expect("error")))).stdout;
        let ro_modem_software_version_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.modem.software.version", cstring_to_string(id).expect("error")))).stdout;
        let ro_kernel_qemu_str = utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop ro.kernel.qemu", cstring_to_string(id).expect("error")))).stdout;
        let no_root_phone_data = NoRootPhoneData {
            kernel_version:kernel_version_str,
            android_version:andord_version_str,
            sdk_version: sdk_version_str,
            ro_product_manufacturer: ro_product_manufacturer_str,
            ro_cpu_abi: ro_cpu_abi_str,
            ro_product_brand: ro_product_brand_str ,
            ro_product_model: ro_product_model_str,
            ro_product_device: ro_product_device_str,
            ro_hardware: ro_hardware_str,
            ro_build_description: ro_build_description_str,
            ro_build_version_security_patch: ro_build_version_security_patch_str,
            ro_build_id: ro_build_id_str,
            ro_bootloader: ro_bootloader_str,
            ro_modem_software_version: ro_modem_software_version_str,
            ro_kernel_qemu: ro_kernel_qemu_str,
        };
        Box::into_raw(Box::new(no_root_phone_data))

    }
    #[no_mangle]
    pub extern "C" fn get_root_phone_data(id:*const c_char)->*mut RootPhoneData{
        let res=utils::exec(utils::str_to_cstr(format!("adb -s {} shell getprop", cstring_to_string(id).expect("error")))).stdout;

        let root_phone_data = RootPhoneData {
            root_ro_serialno: res
        };
        Box::into_raw(Box::new(root_phone_data))

    }

































    #[no_mangle]
    pub extern "C" fn adb_devices_phone(id:*const c_char) -> *const c_char{
        let res = utils::exec(utils::str_to_cstr(format!("adb -s {} devices", cstring_to_string(id).expect("error"))));
        let  r=res.stdout;
        free_command_result(res);
        r
    }
    #[no_mangle]
    pub extern "C" fn fastboot_devices_phone(id:*const c_char) -> *const c_char{
        let res = utils::exec(utils::str_to_cstr(format!("fastboot -s {} devices", cstring_to_string(id).expect("error"))));
        let  r=res.stdout;
        free_command_result(res);
        r
    }
    #[no_mangle]
    pub extern "C" fn get_kernel_version(id:*const c_char) -> *const c_char{
        let res = utils::exec(utils::str_to_cstr(format!("adb -s {} uname -r", cstring_to_string(id).expect("error"))));
        let  r=res.stdout;
        free_command_result(res);
        r
    }


}
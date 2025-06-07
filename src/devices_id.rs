pub mod devices_id {
    use regex::Regex;
    use std::process::Command;
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum HardwareError {
        #[error("Platform not supported")]
        UnsupportedPlatform,

        #[error("Failed to access system information: {0}")]
        AccessDenied(String),

        #[error("Hardware identifier not found")]
        IdNotFound,

        #[error("IO error: {0}")]
        IoError(#[from] std::io::Error),

        #[error("Command execution failed: {0}")]
        CommandFailed(String),

        #[error("Invalid output format")]
        InvalidFormat,
    }

    pub struct HardwareInfo;

    impl HardwareInfo {
        /// 获取跨平台设备唯一标识符
        pub fn get_device_id() -> Result<String, HardwareError> {
            // 优先使用主板ID
            if let Ok(mb_id) = Self::get_motherboard_id() {
                return Ok(mb_id);
            }

            // 其次使用CPU信息+MAC地址的组合
            let cpu_info = Self::get_cpu_info()?;
            let mac_address = Self::get_primary_mac()?;

            // 组合并哈希，确保生成稳定的ID
            let combined = format!("{}{}", cpu_info, mac_address);
            let hashed_id = Self::hash_string(&combined);

            Ok(hashed_id)
        }

        /// 获取主板ID
        pub fn get_motherboard_id() -> Result<String, HardwareError> {
            #[cfg(target_os = "windows")]
            {
                return Self::get_windows_motherboard_id();
            }

            #[cfg(target_os = "linux")]
            {
                return Self::get_linux_motherboard_id();
            }

            #[cfg(target_os = "macos")]
            {
                return Self::get_macos_motherboard_id();
            }

            #[cfg(target_os = "freebsd")]
            {
                return Self::get_freebsd_motherboard_id();
            }

            #[cfg(not(any(
                target_os = "windows",
                target_os = "linux",
                target_os = "macos",
                target_os = "freebsd"
            )))]
            {
                return Err(HardwareError::UnsupportedPlatform);
            }
        }

        /// 获取CPU信息
        pub fn get_cpu_info() -> Result<String, HardwareError> {
            #[cfg(target_os = "windows")]
            {
                let output = Command::new("wmic")
                    .args(&["cpu", "get", "Name"])
                    .output()
                    .map_err(|e| HardwareError::CommandFailed(e.to_string()))?;

                if !output.status.success() {
                    return Err(HardwareError::CommandFailed("WMIC command failed"));
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = stdout.lines().map(|l| l.trim()).collect();

                if lines.len() >= 2 {
                    return Ok(lines[1].to_string());
                }

                Err(HardwareError::IdNotFound)
            }

            #[cfg(target_os = "linux")]
            {
                let content = std::fs::read_to_string("/proc/cpuinfo")?;
                let model_line = content
                    .lines()
                    .find(|l| l.contains("model name"))
                    .ok_or(HardwareError::IdNotFound)?;

                let model = model_line
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim())
                    .ok_or(HardwareError::InvalidFormat)?;

                Ok(model.to_string())
            }

            #[cfg(target_os = "macos")]
            {
                let output = Command::new("sysctl")
                    .args(&["-n", "machdep.cpu.brand_string"])
                    .output()
                    .map_err(|e| HardwareError::CommandFailed(e.to_string()))?;

                if !output.status.success() {
                    return Err(HardwareError::CommandFailed("sysctl command failed"));
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(stdout.trim().to_string())
            }

            #[cfg(target_os = "freebsd")]
            {
                let output = Command::new("sysctl")
                    .args(&["-n", "hw.model"])
                    .output()
                    .map_err(|e| HardwareError::CommandFailed(e.to_string()))?;

                if !output.status.success() {
                    return Err(HardwareError::CommandFailed("sysctl command failed"));
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(stdout.trim().to_string())
            }

            #[cfg(not(any(
                target_os = "windows",
                target_os = "linux",
                target_os = "macos",
                target_os = "freebsd"
            )))]
            {
                Err(HardwareError::UnsupportedPlatform)
            }
        }

        /// 获取主网络接口的MAC地址
        pub fn get_primary_mac() -> Result<String, HardwareError> {
            #[cfg(target_os = "windows")]
            {
                let output = Command::new("getmac")
                    .args(&["/NH", "/FO", "CSV"]) // 无标题，CSV格式
                    .output()
                    .map_err(|e| HardwareError::CommandFailed(e.to_string()))?;

                if !output.status.success() {
                    return Err(HardwareError::CommandFailed("getmac command failed"));
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                let first_line = stdout.lines().next().ok_or(HardwareError::IdNotFound)?;

                // 提取MAC地址（CSV格式的第一个字段）
                let mac = first_line
                    .split(',')
                    .next()
                    .map(|s| s.trim_matches('"').replace("-", ""))
                    .ok_or(HardwareError::InvalidFormat)?;

                Ok(mac)
            }

            #[cfg(target_os = "linux")]
            {
                // 尝试多个网络接口
                let interfaces = vec!["eth0", "wlan0", "enp0s3", "en0"];

                for iface in interfaces {
                    let path = format!("/sys/class/net/{}/address", iface);
                    if let Ok(mac) = std::fs::read_to_string(&path) {
                        let cleaned = mac.trim().replace(":", "");
                        if !cleaned.is_empty() {
                            return Ok(cleaned);
                        }
                    }
                }

                // 如果文件方法失败，尝试使用ip命令
                let output = Command::new("ip")
                    .args(&["link", "show"])
                    .output()
                    .map_err(|e| HardwareError::CommandFailed(e.to_string()))?;

                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let re = Regex::new(r"link/ether\s+([0-9a-fA-F:]+)").unwrap();

                    if let Some(caps) = re.captures(&*stdout) {
                        if let Some(mac) = caps.get(1) {
                            return Ok(mac.as_str().replace(":", ""));
                        }
                    }
                }

                Err(HardwareError::IdNotFound)
            }

            #[cfg(target_os = "macos")]
            {
                let output = Command::new("ifconfig")
                    .arg("en0")
                    .output()
                    .map_err(|e| HardwareError::CommandFailed(e.to_string()))?;

                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let re = Regex::new(r"ether\s+([0-9a-fA-F:]+)").unwrap();

                    if let Some(caps) = re.captures(stdout) {
                        if let Some(mac) = caps.get(1) {
                            return Ok(mac.as_str().replace(":", ""));
                        }
                    }
                }

                // 尝试en1接口
                let output = Command::new("ifconfig")
                    .arg("en1")
                    .output()
                    .map_err(|e| HardwareError::CommandFailed(e.to_string()))?;

                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let re = Regex::new(r"ether\s+([0-9a-fA-F:]+)").unwrap();

                    if let Some(caps) = re.captures(stdout) {
                        if let Some(mac) = caps.get(1) {
                            return Ok(mac.as_str().replace(":", ""));
                        }
                    }
                }

                Err(HardwareError::IdNotFound)
            }

            #[cfg(target_os = "freebsd")]
            {
                let output = Command::new("ifconfig")
                    .arg("em0")
                    .output()
                    .map_err(|e| HardwareError::CommandFailed(e.to_string()))?;

                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let re = Regex::new(r"lladdr\s+([0-9a-fA-F:]+)").unwrap();

                    if let Some(caps) = re.captures(stdout) {
                        if let Some(mac) = caps.get(1) {
                            return Ok(mac.as_str().replace(":", ""));
                        }
                    }
                }

                Err(HardwareError::IdNotFound)
            }

            #[cfg(not(any(
                target_os = "windows",
                target_os = "linux",
                target_os = "macos",
                target_os = "freebsd"
            )))]
            {
                Err(HardwareError::UnsupportedPlatform)
            }
        }

        // Windows平台获取主板ID的具体实现，用cfg标记
        #[cfg(target_os = "windows")]
        fn get_windows_motherboard_id() -> Result<String, HardwareError> {
            use winreg::RegKey;
            use winreg::enums::*;

            // 尝试从注册表获取
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

            // 尝试不同的注册表路径
            let possible_paths = vec![
                "HARDWARE\\DESCRIPTION\\System\\BIOS",
                "SYSTEM\\CurrentControlSet\\Control\\SystemInformation",
            ];

            let possible_values = vec![
                "BaseBoardProduct",
                "SystemProductName",
                "BaseBoardSerialNumber",
                "SystemSerialNumber",
                "BIOSVersion",
            ];

            for path in possible_paths {
                if let Ok(key) = hklm.open_subkey(path) {
                    for value_name in &possible_values {
                        if let Ok(id) = key.get_value::<String, _>(value_name) {
                            let cleaned = id.trim().to_string();
                            if !cleaned.is_empty() && cleaned != "To be filled by O.E.M." {
                                return Ok(cleaned);
                            }
                        }
                    }
                }
            }

            // 如果注册表方法失败，尝试使用WMIC命令
            let output = Command::new("wmic")
                .args(&["csproduct", "get", "uuid"])
                .output()
                .map_err(|e| HardwareError::CommandFailed(e.to_string()))?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = stdout.lines().map(|l| l.trim()).collect();

                if lines.len() >= 2 {
                    let uuid = lines[1].to_string();
                    if !uuid.is_empty() && uuid != "UUID" {
                        return Ok(uuid);
                    }
                }
            }

            Err(HardwareError::IdNotFound)
        }

        // Linux平台获取主板ID的具体实现
        #[cfg(target_os = "linux")]
        fn get_linux_motherboard_id() -> Result<String, HardwareError> {
            // 尝试读取多个可能的DMI信息文件
            let possible_paths = vec![
                "/sys/class/dmi/id/product_uuid",
                "/sys/class/dmi/id/board_serial",
                "/sys/class/dmi/id/product_serial",
                "/sys/class/dmi/id/board_name",
                "/sys/class/dmi/id/board_vendor",
            ];

            for path in possible_paths {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let id = content.trim().to_string();
                    if !id.is_empty() && id != "0" && id != "None" && id != "Default string" {
                        return Ok(id);
                    }
                }
            }

            // 如果文件读取失败，尝试使用dmidecode命令
            let output = Command::new("dmidecode")
                .args(&["-t", "1"]) // 1表示系统信息
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // 尝试提取序列号
                    if let Some(line) = stdout.lines().find(|l| l.contains("Serial Number:")) {
                        let parts: Vec<&str> = line.splitn(2, ':').collect();
                        if parts.len() > 1 {
                            let serial = parts[1].trim().to_string();
                            if !serial.is_empty() && serial != "None" {
                                return Ok(serial);
                            }
                        }
                    }

                    // 尝试提取UUID
                    if let Some(line) = stdout.lines().find(|l| l.contains("UUID:")) {
                        let parts: Vec<&str> = line.splitn(2, ':').collect();
                        if parts.len() > 1 {
                            let uuid = parts[1].trim().to_string();
                            if !uuid.is_empty() && uuid != "00000000-0000-0000-0000-000000000000" {
                                return Ok(uuid);
                            }
                        }
                    }
                }
            }

            // 最后尝试使用lshw命令
            let output = Command::new("lshw").args(&["-C", "system"]).output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // 尝试提取序列号
                    if let Some(line) = stdout.lines().find(|l| l.contains("serial:")) {
                        let parts: Vec<&str> = line.splitn(2, ':').collect();
                        if parts.len() > 1 {
                            let serial = parts[1].trim().to_string();
                            if !serial.is_empty() && serial != "None" {
                                return Ok(serial);
                            }
                        }
                    }
                }
            }

            Err(HardwareError::IdNotFound)
        }

        // macOS平台获取主板ID的具体实现
        #[cfg(target_os = "macos")]
        fn get_macos_motherboard_id() -> Result<String, HardwareError> {
            // 使用system_profiler命令获取硬件信息
            let output = Command::new("system_profiler")
                .args(&["SPHardwareDataType", "-detailLevel", "full"])
                .output()
                .map_err(|e| HardwareError::CommandFailed(e.to_string()))?;

            if !output.status.success() {
                return Err(HardwareError::CommandFailed(
                    "system_profiler command failed",
                ));
            }

            let stdout = String::from_utf8_lossy(&output.stdout);

            // 尝试提取逻辑板序列号
            if let Some(line) = stdout
                .lines()
                .find(|l| l.contains("Logic Board Serial Number:"))
            {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() > 1 {
                    let serial = parts[1].trim().to_string();
                    if !serial.is_empty() {
                        return Ok(serial);
                    }
                }
            }

            // 尝试提取硬件UUID
            if let Some(line) = stdout.lines().find(|l| l.contains("Hardware UUID:")) {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() > 1 {
                    let uuid = parts[1].trim().to_string();
                    if !uuid.is_empty() {
                        return Ok(uuid);
                    }
                }
            }

            // 尝试提取主板型号
            if let Some(line) = stdout.lines().find(|l| l.contains("Model Identifier:")) {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() > 1 {
                    let model = parts[1].trim().to_string();
                    if !model.is_empty() {
                        return Ok(model);
                    }
                }
            }

            Err(HardwareError::IdNotFound)
        }

        // FreeBSD平台获取主板ID的具体实现
        #[cfg(target_os = "freebsd")]
        fn get_freebsd_motherboard_id() -> Result<String, HardwareError> {
            // 使用dmidecode命令（如果安装）
            let output = Command::new("dmidecode").args(&["-t", "1"]).output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // 尝试提取序列号
                    if let Some(line) = stdout.lines().find(|l| l.contains("Serial Number:")) {
                        let parts: Vec<&str> = line.splitn(2, ':').collect();
                        if parts.len() > 1 {
                            let serial = parts[1].trim().to_string();
                            if !serial.is_empty() && serial != "None" {
                                return Ok(serial);
                            }
                        }
                    }
                }
            }

            // 尝试使用pciconf命令
            let output = Command::new("pciconf")
                .args(&["-lv"])
                .output()
                .map_err(|e| HardwareError::CommandFailed(e.to_string()))?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // 尝试提取主板信息
                if let Some(chunk) = stdout.split("\n\n").find(|c| c.contains("class=0x060000")) {
                    if let Some(line) = chunk.lines().find(|l| l.contains("vendor=")) {
                        if let Some(line2) = chunk.lines().find(|l| l.contains("device=")) {
                            let vendor = line.split('=').nth(1).unwrap_or("").trim_matches('"');
                            let device = line2.split('=').nth(1).unwrap_or("").trim_matches('"');
                            let combined = format!("{}_{}", vendor, device);

                            if !combined.is_empty() {
                                return Ok(combined);
                            }
                        }
                    }
                }
            }

            Err(HardwareError::IdNotFound)
        }

        /// 哈希字符串生成固定长度的标识符
        fn hash_string(s: &str) -> String {
            use sha2::{Digest, Sha256};

            let mut hasher = Sha256::new();
            hasher.update(s.as_bytes());
            let result = hasher.finalize();

            hex::encode(result)
        }
    }

    // 测试代码
    #[test]
    fn main() {
        match HardwareInfo::get_device_id() {
            Ok(id) => println!("Device ID: {}", id),
            Err(e) => eprintln!("Error getting device ID: {}", e),
        }

        match HardwareInfo::get_motherboard_id() {
            Ok(id) => println!("Motherboard ID: {}", id),
            Err(e) => eprintln!("Error getting motherboard ID: {}", e),
        }

        match HardwareInfo::get_cpu_info() {
            Ok(info) => println!("CPU Info: {}", info),
            Err(e) => eprintln!("Error getting CPU info: {}", e),
        }

        match HardwareInfo::get_primary_mac() {
            Ok(mac) => println!("Primary MAC: {}", mac),
            Err(e) => eprintln!("Error getting MAC address: {}", e),
        }
    }
}

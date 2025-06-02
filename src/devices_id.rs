use thiserror::Error;
use sha2::{Sha256, Digest};
use hex;

#[derive(Error, Debug)]
pub enum DeviceIdError {
    #[error("Failed to get device identifier")]
    IdentifierUnavailable,
    #[error("Permission denied when accessing device information")]
    PermissionDenied,
    #[error("Platform not supported")]
    UnsupportedPlatform,
}

/// 设备标识符获取器
pub struct DeviceIdentifier;

impl DeviceIdentifier {
    /// 获取最佳可用设备标识符
    pub fn get() -> Result<String, DeviceIdError> {
        Self::get_motherboard_id()
            .or_else(|_| Self::get_fallback_id())
    }

    /// 获取主板标识符（平台特定）
    pub fn get_motherboard_id() -> Result<String, DeviceIdError> {
        #[cfg(target_os = "windows")]
        return windows::get_motherboard_id();

        #[cfg(target_os = "linux")]
        return linux::get_motherboard_id();

        #[cfg(target_os = "macos")]
        return macos::get_motherboard_id();

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        Err(DeviceIdError::UnsupportedPlatform)
    }

    /// 获取备用设备ID（基于多种硬件特征的哈希）
    pub fn get_fallback_id() -> Result<String, DeviceIdError> {
        let mut hasher = Sha256::new();

        // 尝试添加主板信息
        if let Ok(mb_id) = Self::get_motherboard_id() {
            hasher.update(mb_id);
        }

        // 添加CPU信息
        if let Ok(cpu_info) = Self::get_cpu_info() {
            hasher.update(cpu_info);
        }

        // 添加MAC地址
        if let Ok(mac) = Self::get_primary_mac() {
            hasher.update(mac);
        }

        // 添加内存信息
        if let Ok(mem_info) = Self::get_mem_info() {
            hasher.update(mem_info);
        }

        // 转换为十六进制字符串
        Ok(hex::encode(hasher.finalize()))
    }

    fn get_cpu_info() -> Result<String, DeviceIdError> {
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/proc/cpuinfo")
                .map_err(|_| DeviceIdError::IdentifierUnavailable)
        }

        #[cfg(target_os = "windows")]
        {
            Ok(windows::get_cpu_info()
                .unwrap_or_else(|| "unknown-cpu".to_string()))
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let output = Command::new("sysctl")
                .arg("-n")
                .arg("machdep.cpu.brand_string")
                .output()
                .map_err(|_| DeviceIdError::PermissionDenied)?;

            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                Err(DeviceIdError::IdentifierUnavailable)
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        Err(DeviceIdError::UnsupportedPlatform)
    }

    fn get_primary_mac() -> Result<String, DeviceIdError> {
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/sys/class/net/eth0/address")
                .or_else(|_| std::fs::read_to_string("/sys/class/net/wlan0/address"))
                .map(|s| s.trim().to_string())
                .map_err(|_| DeviceIdError::IdentifierUnavailable)
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            let output = Command::new("getmac")
                .output()
                .map_err(|_| DeviceIdError::PermissionDenied)?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines()
                    .nth(3) // 通常第一个非标题MAC地址在第4行
                    .and_then(|l| l.split_whitespace().next())
                    .map(|s| s.to_string())
                    .ok_or(DeviceIdError::IdentifierUnavailable)
            } else {
                Err(DeviceIdError::IdentifierUnavailable)
            }
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let output = Command::new("ifconfig")
                .arg("en0")
                .output()
                .map_err(|_| DeviceIdError::PermissionDenied)?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines()
                    .find(|l| l.contains("ether"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .map(|s| s.to_string())
                    .ok_or(DeviceIdError::IdentifierUnavailable)
            } else {
                Err(DeviceIdError::IdentifierUnavailable)
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        Err(DeviceIdError::UnsupportedPlatform)
    }

    fn get_mem_info() -> Result<String, DeviceIdError> {
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/proc/meminfo")
                .map_err(|_| DeviceIdError::IdentifierUnavailable)
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            let output = Command::new("wmic")
                .arg("memorychip")
                .arg("get")
                .arg("SerialNumber")
                .output()
                .map_err(|_| DeviceIdError::PermissionDenied)?;

            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                Err(DeviceIdError::IdentifierUnavailable)
            }
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let output = Command::new("system_profiler")
                .arg("SPMemoryDataType")
                .output()
                .map_err(|_| DeviceIdError::PermissionDenied)?;

            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                Err(DeviceIdError::IdentifierUnavailable)
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        Err(DeviceIdError::UnsupportedPlatform)
    }
}

// Windows 专用实现
#[cfg(target_os = "windows")]
mod windows {
    use super::DeviceIdError;
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    pub fn get_motherboard_id() -> Result<String, DeviceIdError> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let subkey = hklm.open_subkey("HARDWARE\\DESCRIPTION\\System\\BIOS")
            .map_err(|_| DeviceIdError::PermissionDenied)?;

        subkey.get_value("SystemProductName")
            .or_else(|_| subkey.get_value("BaseBoardProduct"))
            .map_err(|_| DeviceIdError::IdentifierUnavailable)
    }

    pub fn get_cpu_info() -> Option<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let subkey = hklm.open_subkey("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0").ok()?;
        subkey.get_value("ProcessorNameString").ok()
    }
}

// Linux 专用实现
#[cfg(target_os = "linux")]
mod linux {
    use super::DeviceIdError;
    use std::fs;

    pub fn get_motherboard_id() -> Result<String, DeviceIdError> {
        fs::read_to_string("/sys/class/dmi/id/product_uuid")
            .or_else(|_| fs::read_to_string("/sys/class/dmi/id/board_serial"))
            .or_else(|_| fs::read_to_string("/sys/class/dmi/id/product_serial"))
            .map(|s| s.trim().to_string())
            .map_err(|_| DeviceIdError::IdentifierUnavailable)
    }
}

// macOS 专用实现
#[cfg(target_os = "macos")]
mod macos {
    use super::DeviceIdError;
    use std::process::Command;

    pub fn get_motherboard_id() -> Result<String, DeviceIdError> {
        let output = Command::new("system_profiler")
            .arg("SPHardwareDataType")
            .output()
            .map_err(|_| DeviceIdError::PermissionDenied)?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines()
                .find(|l| l.contains("Serial Number") || l.contains("Hardware UUID"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_string())
                .ok_or(DeviceIdError::IdentifierUnavailable)
        } else {
            Err(DeviceIdError::IdentifierUnavailable)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_id_generation() {
        let id = DeviceIdentifier::get();
        assert!(id.is_ok());
        let id_str = id.unwrap();
        assert!(!id_str.is_empty());
        println!("Generated Device ID: {}", id_str);

        // 验证ID格式（SHA256应该是64字符的十六进制）
        assert_eq!(id_str.len(), 64);
        assert!(id_str.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
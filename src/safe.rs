pub mod safe {
    use base64::{engine::general_purpose, Engine as _};
    use openssl::{
        error::ErrorStack,
        rand,
        symm::{Cipher, Crypter, Mode},
    };
    use std::ffi::{c_char, CStr, CString};
    use thiserror::Error;

    // 常量定义（保持不变）
    pub const AES_128_KEY_LEN: usize = 16;
    pub const AES_192_KEY_LEN: usize = 24;
    pub const AES_256_KEY_LEN: usize = 32;
    pub const DEFAULT_NONCE_LEN: usize = 12; // GCM推荐的IV长度（12字节）
    pub const DEFAULT_TAG_LEN: usize = 16; // GCM认证标签固定长度（16字节）
    pub const AES_BLOCK_SIZE: usize = 16; // AES块大小（16字节）

    // 错误定义（保持不变）
    #[derive(Debug, Error)]
    pub enum KeyError {
        #[error("随机密钥生成失败")]
        RandomFailed,
        #[error(
            "密钥长度无效: 必须为 {AES_128_KEY_LEN}, {AES_192_KEY_LEN} 或 {AES_256_KEY_LEN} 字节"
        )]
        InvalidKeyLength,
    }

    #[derive(Debug, Error)]
    pub enum CryptoError {
        #[error("加密失败: {0}")]
        EncryptionFailed(String),
        #[error("解密失败: {0}")]
        DecryptionFailed(String),
        #[error(
            "不支持的密钥长度: 需要 {AES_128_KEY_LEN}, {AES_192_KEY_LEN} 或 {AES_256_KEY_LEN} 字节，实际 {actual} 字节"
        )]
        UnsupportedKeyLength { actual: usize },
        #[error("密文长度无效: 最小长度应为 {min_length} 字节，实际为 {actual} 字节")]
        InvalidCiphertextLength { min_length: usize, actual: usize },
        #[error("认证标签验证失败")]
        TagVerificationFailed,
        #[error("IV/Nonce长度无效")]
        InvalidNonceLength,
        #[error("认证标签长度无效")]
        InvalidTagLength,
        #[error("Base64编码失败: {0}")]
        Base64EncodeError(String),
        #[error("Base64解码失败: {0}")]
        Base64DecodeError(String),
        #[error("密文格式错误")]
        InvalidCiphertextFormat,
        #[error("UTF-8解码失败: {0}")]
        Utf8DecodingFailed(String),
    }

    impl From<ErrorStack> for CryptoError {
        fn from(err: ErrorStack) -> Self {
            CryptoError::EncryptionFailed(err.to_string())
        }
    }

    // C接口错误码定义
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CryptoErrorCode {
        Success = 0,
        EncryptionFailed = 1,
        DecryptionFailed = 2,
        UnsupportedKeyLength = 3,
        InvalidCiphertextLength = 4,
        TagVerificationFailed = 5,
        InvalidNonceLength = 6,
        InvalidTagLength = 7,
        Base64EncodeError = 8,
        Base64DecodeError = 9,
        InvalidCiphertextFormat = 10,
        Utf8DecodingFailed = 11,
        KeyGenerationFailed = 12,
        NullPointerError = 13,
    }

    // C接口结构体：加密解密选项
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct CEncryptionOptions {
        pub nonce_length: usize,
        pub tag_length: usize,
    }

    // C接口结构体：AES-CBC模式
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CAesCbcMode {
        FixedIv = 0,
        RandomIv = 1,
    }

    // 密钥生成（保持原逻辑）
    pub fn generate_key<const N: usize>() -> Result<[u8; N], KeyError> {
        if N != AES_128_KEY_LEN && N != AES_192_KEY_LEN && N != AES_256_KEY_LEN {
            return Err(KeyError::InvalidKeyLength);
        }

        let mut key = [0u8; N];
        rand::rand_bytes(&mut key).map_err(|_| KeyError::RandomFailed)?;
        Ok(key)
    }

    pub fn generate_key_base64<const N: usize>() -> Result<String, KeyError> {
        let key = generate_key::<N>()?;
        Ok(base64_encode(&key))
    }

    // Base64编解码（保持原逻辑）
    pub fn base64_encode(data: &[u8]) -> String {
        general_purpose::STANDARD.encode(data)
    }

    pub fn base64_decode(data: &str) -> Result<Vec<u8>, CryptoError> {
        general_purpose::STANDARD
            .decode(data)
            .map_err(|e| CryptoError::Base64DecodeError(e.to_string()))
    }

    // GCM模式核心逻辑（保持原逻辑）
    fn select_cipher(key: &[u8]) -> Result<Cipher, CryptoError> {
        match key.len() {
            AES_128_KEY_LEN => Ok(Cipher::aes_128_gcm()),
            AES_192_KEY_LEN => Ok(Cipher::aes_192_gcm()),
            AES_256_KEY_LEN => Ok(Cipher::aes_256_gcm()),
            len => Err(CryptoError::UnsupportedKeyLength { actual: len }),
        }
    }

    #[derive(Debug, Clone)]
    pub struct EncryptionOptions {
        pub nonce_length: usize,
        pub tag_length: usize,
    }

    impl Default for EncryptionOptions {
        fn default() -> Self {
            Self {
                nonce_length: DEFAULT_NONCE_LEN,
                tag_length: DEFAULT_TAG_LEN,
            }
        }
    }

    pub fn encrypt_with_options(
        key: &[u8],
        plaintext: &[u8],
        options: &EncryptionOptions,
    ) -> Result<Vec<u8>, CryptoError> {
        if options.nonce_length == 0 || options.tag_length == 0 {
            return Err(CryptoError::InvalidNonceLength);
        }

        let cipher = select_cipher(key)?;
        let mut iv = vec![0u8; options.nonce_length];
        rand::rand_bytes(&mut iv)
            .map_err(|err: ErrorStack| CryptoError::EncryptionFailed(err.to_string()))?;

        let mut encrypter = Crypter::new(cipher, Mode::Encrypt, key, Some(&iv))?;
        encrypter.pad(false);

        let mut ciphertext = Vec::new();
        encrypter.update(plaintext, &mut ciphertext)?;
        encrypter.finalize(&mut ciphertext)?;

        let mut tag = vec![0u8; options.tag_length];
        encrypter.get_tag(&mut tag)?;

        let mut result = Vec::new();
        result.extend(&iv);
        result.extend(&ciphertext);
        result.extend(&tag);

        Ok(result)
    }

    pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        encrypt_with_options(key, plaintext, &EncryptionOptions::default())
    }

    pub fn encrypt_to_base64(key: &[u8], plaintext: &[u8]) -> Result<String, CryptoError> {
        let ciphertext = encrypt(key, plaintext)?;
        Ok(base64_encode(&ciphertext))
    }

    #[derive(Debug, Clone)]
    pub struct DecryptionOptions {
        pub nonce_length: usize,
        pub tag_length: usize,
    }

    impl Default for DecryptionOptions {
        fn default() -> Self {
            Self {
                nonce_length: DEFAULT_NONCE_LEN,
                tag_length: DEFAULT_TAG_LEN,
            }
        }
    }

    pub fn decrypt_with_options(
        key: &[u8],
        ciphertext: &[u8],
        options: &DecryptionOptions,
    ) -> Result<Vec<u8>, CryptoError> {
        if options.nonce_length == 0 || options.tag_length == 0 {
            return Err(CryptoError::InvalidNonceLength);
        }

        let cipher = select_cipher(key)?;
        let min_length = options.nonce_length + options.tag_length;
        if ciphertext.len() < min_length {
            return Err(CryptoError::InvalidCiphertextLength {
                min_length,
                actual: ciphertext.len(),
            });
        }

        let (iv, rest) = ciphertext.split_at(options.nonce_length);
        let (cipher_data, tag) = rest.split_at(rest.len() - options.tag_length);

        let mut decrypter = Crypter::new(cipher, Mode::Decrypt, key, Some(iv))?;
        decrypter.pad(false);
        decrypter.set_tag(tag)?;

        let mut plaintext = Vec::new();
        decrypter.update(cipher_data, &mut plaintext)?;
        decrypter.finalize(&mut plaintext)?;

        Ok(plaintext)
    }

    pub fn decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        decrypt_with_options(key, ciphertext, &DecryptionOptions::default())
    }

    pub fn decrypt_from_base64(
        key: &[u8],
        ciphertext_base64: &str,
    ) -> Result<Vec<u8>, CryptoError> {
        let ciphertext = base64_decode(ciphertext_base64)?;
        decrypt(key, &ciphertext)
    }

    // AES-CBC-192 模式（保持原逻辑）
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AesCbcMode {
        FixedIv,
        RandomIv,
    }

    pub fn encrypt_cbc_192(
        key: &[u8],
        plaintext: &str,
        mode: AesCbcMode,
    ) -> Result<String, CryptoError> {
        if key.len() != AES_192_KEY_LEN {
            return Err(CryptoError::UnsupportedKeyLength { actual: key.len() });
        }

        let iv = match mode {
            AesCbcMode::FixedIv => vec![0u8; AES_BLOCK_SIZE],
            AesCbcMode::RandomIv => {
                let mut iv = vec![0u8; AES_BLOCK_SIZE];
                rand::rand_bytes(&mut iv)
                    .map_err(|e| CryptoError::EncryptionFailed(format!("生成随机IV失败: {}", e)))?;
                iv
            }
        };

        let cipher = Cipher::aes_192_cbc();
        let mut encrypter = Crypter::new(cipher, Mode::Encrypt, key, Some(&iv))?;
        encrypter.pad(true);

        let mut ciphertext = Vec::new();
        encrypter.update(plaintext.as_bytes(), &mut ciphertext)?;
        encrypter.finalize(&mut ciphertext)?;

        match mode {
            AesCbcMode::FixedIv => Ok(base64_encode(&ciphertext)),
            AesCbcMode::RandomIv => {
                let iv_b64 = base64_encode(&iv);
                let ciphertext_b64 = base64_encode(&ciphertext);
                Ok(format!("R|{}|{}", iv_b64, ciphertext_b64))
            }
        }
    }

    pub fn decrypt_cbc_192(key: &[u8], ciphertext_base64: &str) -> Result<String, CryptoError> {
        if key.len() != AES_192_KEY_LEN {
            return Err(CryptoError::UnsupportedKeyLength { actual: key.len() });
        }

        if ciphertext_base64.starts_with("R|") {
            let parts: Vec<&str> = ciphertext_base64.splitn(3, '|').collect();
            if parts.len() != 3 {
                return Err(CryptoError::InvalidCiphertextFormat);
            }

            let iv_b64 = parts[1];
            let ciphertext_b64 = parts[2];
            let iv = base64_decode(iv_b64)?;
            let ciphertext = base64_decode(ciphertext_b64)?;

            if iv.len() != AES_BLOCK_SIZE {
                return Err(CryptoError::InvalidNonceLength);
            }

            let cipher = Cipher::aes_192_cbc();
            let mut decrypter = Crypter::new(cipher, Mode::Decrypt, key, Some(&iv))?;
            decrypter.pad(true);

            let mut plaintext = Vec::new();
            decrypter.update(&ciphertext, &mut plaintext)?;
            decrypter.finalize(&mut plaintext)?;

            String::from_utf8(plaintext).map_err(|e| CryptoError::Utf8DecodingFailed(e.to_string()))
        } else {
            let ciphertext = base64_decode(ciphertext_base64)?;
            let iv = [0u8; AES_BLOCK_SIZE];

            let cipher = Cipher::aes_192_cbc();
            let mut decrypter = Crypter::new(cipher, Mode::Decrypt, key, Some(&iv))?;
            decrypter.pad(true);

            let mut plaintext = Vec::new();
            decrypter.update(&ciphertext, &mut plaintext)?;
            decrypter.finalize(&mut plaintext)?;

            String::from_utf8(plaintext).map_err(|e| CryptoError::Utf8DecodingFailed(e.to_string()))
        }
    }

    // C接口辅助函数：错误转换
    fn crypto_error_to_code(err: &CryptoError) -> CryptoErrorCode {
        match err {
            CryptoError::EncryptionFailed(_) => CryptoErrorCode::EncryptionFailed,
            CryptoError::DecryptionFailed(_) => CryptoErrorCode::DecryptionFailed,
            CryptoError::UnsupportedKeyLength { .. } => CryptoErrorCode::UnsupportedKeyLength,
            CryptoError::InvalidCiphertextLength { .. } => CryptoErrorCode::InvalidCiphertextLength,
            CryptoError::TagVerificationFailed => CryptoErrorCode::TagVerificationFailed,
            CryptoError::InvalidNonceLength => CryptoErrorCode::InvalidNonceLength,
            CryptoError::InvalidTagLength => CryptoErrorCode::InvalidTagLength,
            CryptoError::Base64EncodeError(_) => CryptoErrorCode::Base64EncodeError,
            CryptoError::Base64DecodeError(_) => CryptoErrorCode::Base64DecodeError,
            CryptoError::InvalidCiphertextFormat => CryptoErrorCode::InvalidCiphertextFormat,
            CryptoError::Utf8DecodingFailed(_) => CryptoErrorCode::Utf8DecodingFailed,
        }
    }

    // C接口：生成AES密钥（128位）
   #[unsafe(no_mangle)]
    pub extern "C" fn generate_aes128_key(key_buf: *mut u8, key_len: *mut usize) -> CryptoErrorCode {
        if key_buf.is_null() || key_len.is_null() {
            return CryptoErrorCode::NullPointerError;
        }

        let key = match generate_key::<AES_128_KEY_LEN>() {
            Ok(k) => k,
            Err(_) => return CryptoErrorCode::KeyGenerationFailed,
        };

        unsafe {
            *key_len = AES_128_KEY_LEN;
            let dest = std::slice::from_raw_parts_mut(key_buf, AES_128_KEY_LEN);
            dest.copy_from_slice(&key);
        }
        CryptoErrorCode::Success
    }

    // C接口：生成AES密钥（192位）
   #[unsafe(no_mangle)]
    pub extern "C" fn generate_aes192_key(key_buf: *mut u8, key_len: *mut usize) -> CryptoErrorCode {
        if key_buf.is_null() || key_len.is_null() {
            return CryptoErrorCode::NullPointerError;
        }

        let key = match generate_key::<AES_192_KEY_LEN>() {
            Ok(k) => k,
            Err(_) => return CryptoErrorCode::KeyGenerationFailed,
        };

        unsafe {
            *key_len = AES_192_KEY_LEN;
            let dest = std::slice::from_raw_parts_mut(key_buf, AES_192_KEY_LEN);
            dest.copy_from_slice(&key);
        }
        CryptoErrorCode::Success
    }

    // C接口：生成AES密钥（256位，Base64编码）
   #[unsafe(no_mangle)]
    pub extern "C" fn generate_aes256_key_base64(out_key: *mut *mut c_char) -> CryptoErrorCode {
        if out_key.is_null() {
            return CryptoErrorCode::NullPointerError;
        }

        let key_str = match generate_key_base64::<AES_256_KEY_LEN>() {
            Ok(s) => s,
            Err(_) => return CryptoErrorCode::KeyGenerationFailed,
        };

        let c_str = match CString::new(key_str) {
            Ok(s) => s,
            Err(_) => return CryptoErrorCode::Base64EncodeError,
        };

        unsafe {
            *out_key = c_str.into_raw();
        }
        CryptoErrorCode::Success
    }

    // C接口：AES-GCM加密（Base64输出）
   #[unsafe(no_mangle)]
    pub extern "C" fn aes_gcm_encrypt_base64(
        key: *const u8,
        key_len: usize,
        plaintext: *const c_char,
        ciphertext_out: *mut *mut c_char
    ) -> CryptoErrorCode {
        if key.is_null() || plaintext.is_null() || ciphertext_out.is_null() {
            return CryptoErrorCode::NullPointerError;
        }

        let key_slice = unsafe { std::slice::from_raw_parts(key, key_len) };
        let plaintext_str = unsafe { CStr::from_ptr(plaintext).to_string_lossy().into_owned() };

        let result = encrypt_to_base64(key_slice, plaintext_str.as_bytes());
        match result {
            Ok(ciphertext) => {
                let c_str = match CString::new(ciphertext) {
                    Ok(s) => s,
                    Err(_) => return CryptoErrorCode::Base64EncodeError,
                };
                unsafe { *ciphertext_out = c_str.into_raw() };
                CryptoErrorCode::Success
            }
            Err(e) => crypto_error_to_code(&e),
        }
    }

    // C接口：AES-GCM解密（Base64输入）
   #[unsafe(no_mangle)]
    pub extern "C" fn aes_gcm_decrypt_base64(
        key: *const u8,
        key_len: usize,
        ciphertext: *const c_char,
        plaintext_out: *mut *mut c_char
    ) -> CryptoErrorCode {
        if key.is_null() || ciphertext.is_null() || plaintext_out.is_null() {
            return CryptoErrorCode::NullPointerError;
        }

        let key_slice = unsafe { std::slice::from_raw_parts(key, key_len) };
        let ciphertext_str = unsafe { CStr::from_ptr(ciphertext).to_string_lossy().into_owned() };

        let result = decrypt_from_base64(key_slice, &ciphertext_str);
        match result {
            Ok(plaintext_bytes) => {
                let plaintext_str = match String::from_utf8(plaintext_bytes) {
                    Ok(s) => s,
                    Err(_e) => return CryptoErrorCode::Utf8DecodingFailed,
                };
                let c_str = match CString::new(plaintext_str) {
                    Ok(s) => s,
                    Err(_) => return CryptoErrorCode::Base64DecodeError,
                };
                unsafe { *plaintext_out = c_str.into_raw() };
                CryptoErrorCode::Success
            }
            Err(e) => crypto_error_to_code(&e),
        }
    }

    // C接口：AES-CBC-192加密
   #[unsafe(no_mangle)]
    pub extern "C" fn aes_cbc192_encrypt(
        key: *const u8,
        key_len: usize,
        plaintext: *const c_char,
        mode: CAesCbcMode,
        ciphertext_out: *mut *mut c_char
    ) -> CryptoErrorCode {
        if key.is_null() || plaintext.is_null() || ciphertext_out.is_null() {
            return CryptoErrorCode::NullPointerError;
        }

        if key_len != AES_192_KEY_LEN {
            return CryptoErrorCode::UnsupportedKeyLength;
        }

        let key_slice = unsafe { std::slice::from_raw_parts(key, key_len) };
        let plaintext_str = unsafe { CStr::from_ptr(plaintext).to_string_lossy().into_owned() };
        let rust_mode = match mode {
            CAesCbcMode::FixedIv => AesCbcMode::FixedIv,
            CAesCbcMode::RandomIv => AesCbcMode::RandomIv,
        };

        let result = encrypt_cbc_192(key_slice, &plaintext_str, rust_mode);
        match result {
            Ok(ciphertext) => {
                let c_str = match CString::new(ciphertext) {
                    Ok(s) => s,
                    Err(_) => return CryptoErrorCode::Base64EncodeError,
                };
                unsafe { *ciphertext_out = c_str.into_raw() };
                CryptoErrorCode::Success
            }
            Err(e) => crypto_error_to_code(&e),
        }
    }

    // C接口：AES-CBC-192解密
   #[unsafe(no_mangle)]
    pub extern "C" fn aes_cbc192_decrypt(
        key: *const u8,
        key_len: usize,
        ciphertext: *const c_char,
        plaintext_out: *mut *mut c_char
    ) -> CryptoErrorCode {
        if key.is_null() || ciphertext.is_null() || plaintext_out.is_null() {
            return CryptoErrorCode::NullPointerError;
        }

        if key_len != AES_192_KEY_LEN {
            return CryptoErrorCode::UnsupportedKeyLength;
        }

        let key_slice = unsafe { std::slice::from_raw_parts(key, key_len) };
        let ciphertext_str = unsafe { CStr::from_ptr(ciphertext).to_string_lossy().into_owned() };

        let result = decrypt_cbc_192(key_slice, &ciphertext_str);
        match result {
            Ok(plaintext) => {
                let c_str = match CString::new(plaintext) {
                    Ok(s) => s,
                    Err(e) => return crypto_error_to_code(&CryptoError::Utf8DecodingFailed(e.to_string())),
                };
                unsafe { *plaintext_out = c_str.into_raw() };
                CryptoErrorCode::Success
            }
            Err(e) => crypto_error_to_code(&e),
        }
    }

    // C接口：Base64编码
   #[unsafe(no_mangle)]
    pub extern "C" fn base64_encode_c(
        data: *const u8,
        data_len: usize,
        out_str: *mut *mut c_char
    ) -> CryptoErrorCode {
        if data.is_null() || out_str.is_null() {
            return CryptoErrorCode::NullPointerError;
        }

        let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };
        let encoded = base64_encode(data_slice);
        let c_str = match CString::new(encoded) {
            Ok(s) => s,
            Err(_) => return CryptoErrorCode::Base64EncodeError,
        };

        unsafe { *out_str = c_str.into_raw() };
        CryptoErrorCode::Success
    }

    // C接口：Base64解码
   #[unsafe(no_mangle)]
    pub extern "C" fn base64_decode_c(
        data_str: *const c_char,
        out_data: *mut *mut u8,
        out_len: *mut usize
    ) -> CryptoErrorCode {
        if data_str.is_null() || out_data.is_null() || out_len.is_null() {
            return CryptoErrorCode::NullPointerError;
        }

        let data = unsafe { CStr::from_ptr(data_str).to_string_lossy().into_owned() };
        let decoded = match base64_decode(&data) {
            Ok(d) => d,
            Err(e) => return crypto_error_to_code(&e),
        };

        unsafe {
            *out_len = decoded.len();
            let mut buf = vec![0u8; decoded.len()].into_boxed_slice();
            buf.copy_from_slice(&decoded);
            *out_data = buf.as_mut_ptr();
            std::mem::forget(buf); // 转移所有权给C端
        }
        CryptoErrorCode::Success
    }

    // C接口：释放C字符串
   #[unsafe(no_mangle)]
    pub extern "C" fn free_c_string(s: *mut c_char) {
        unsafe {
            if !s.is_null() {
                let _ = CString::from_raw(s);
            }
        }
    }

    // C接口：释放字节缓冲区
   #[unsafe(no_mangle)]
    pub extern "C" fn free_byte_buffer(buf: *mut u8, len: usize) {
        unsafe {
            if !buf.is_null() {
                let _ = Vec::from_raw_parts(buf, len, len);
            }
        }
    }

    // 测试代码（保持不变）
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_gcm_encryption_decryption() {
            let key = generate_key::<AES_256_KEY_LEN>().unwrap();
            let plaintext = b"Hello, GCM!";

            let ciphertext = encrypt(&key, plaintext).unwrap();
            let decrypted = decrypt(&key, &ciphertext).unwrap();
            assert_eq!(plaintext, decrypted.as_slice());
        }

        #[test]
        fn test_cbc_192_fixed_iv() {
            let key = generate_key::<AES_192_KEY_LEN>().unwrap();
            let plaintext = "Fixed IV Test";

            let ciphertext = encrypt_cbc_192(&key, plaintext, AesCbcMode::FixedIv).unwrap();
            let decrypted = decrypt_cbc_192(&key, &ciphertext).unwrap();
            assert_eq!(plaintext, decrypted);
        }

        #[test]
        fn test_cbc_192_random_iv() {
            let key = generate_key::<AES_192_KEY_LEN>().unwrap();
            let plaintext = "Random IV Test";

            let ciphertext = encrypt_cbc_192(&key, plaintext, AesCbcMode::RandomIv).unwrap();
            let decrypted = decrypt_cbc_192(&key, &ciphertext).unwrap();
            assert_eq!(plaintext, decrypted);
        }

        #[test]
        fn test_invalid_key_length() {
            let key = "shortkey".as_bytes(); // 10字节
            let plaintext = "Test";

            assert!(encrypt_cbc_192(key, plaintext, AesCbcMode::FixedIv).is_err());
        }
    }
}
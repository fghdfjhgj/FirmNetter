pub mod safe {
    use base64::{Engine as _, engine::general_purpose};
    use openssl::{
        error::ErrorStack,
        rand,
        symm::{Cipher, Crypter, Mode},
    };
    use thiserror::Error;

    // 常量定义
    pub const AES_128_KEY_LEN: usize = 16;
    pub const AES_192_KEY_LEN: usize = 24;
    pub const AES_256_KEY_LEN: usize = 32;
    pub const DEFAULT_NONCE_LEN: usize = 12; // GCM推荐的IV长度（12字节）
    pub const DEFAULT_TAG_LEN: usize = 16; // GCM认证标签固定长度（16字节）
    pub const AES_BLOCK_SIZE: usize = 16; // AES块大小（16字节）

    // 错误定义
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

    // 密钥生成
    /// 生成指定长度的AES加密密钥。
    ///
    /// 此函数根据常量泛型参数N指定的长度生成一个随机的AES加密密钥。
    /// 它首先检查所需密钥长度是否符合AES加密标准（128、192或256位），
    /// 如果不符合，则返回一个错误。如果密钥长度有效，则生成相应长度的随机密钥。
    ///
    /// # 类型参数
    /// - `N`: 一个常量 usize 值，指定所需密钥的长度（以字节为单位）。
    ///
    /// # 返回值
    /// - `Ok([u8; N])`: 当密钥成功生成时，返回一个长度为N的字节数组。
    /// - `Err(KeyError)`: 当密钥长度无效或随机密钥生成失败时，返回一个`KeyError`。
    pub fn generate_key<const N: usize>() -> Result<[u8; N], KeyError> {
        // 检查密钥长度是否为AES加密标准支持的长度
        if N != AES_128_KEY_LEN && N != AES_192_KEY_LEN && N != AES_256_KEY_LEN {
            return Err(KeyError::InvalidKeyLength);
        }

        // 初始化一个长度为N的字节数组来存储密钥
        let mut key = [0u8; N];
        // 生成随机密钥，如果生成失败则返回错误
        rand::rand_bytes(&mut key).map_err(|_| KeyError::RandomFailed)?;
        // 成功生成密钥后，返回Ok
        Ok(key)
    }

    /// 生成一个Base64编码的随机密钥
    ///
    /// # 泛型参数
    /// - `N`: 密钥的长度，由调用者指定
    ///
    /// # 返回值
    /// - `Result<String, KeyError>`: 返回一个结果，包含生成的Base64编码密钥字符串或错误信息
    ///
    /// # 功能描述
    /// 本函数旨在生成一个指定长度的随机密钥，并将其Base64编码后返回
    /// 它首先调用`generate_key`函数生成一个二进制密钥，然后使用`base64_encode`函数将其编码为Base64格式的字符串
    /// 如果在生成密钥或编码过程中遇到错误，将返回相应的错误信息
    pub fn generate_key_base64<const N: usize>() -> Result<String, KeyError> {
        // 生成一个长度为N的二进制密钥
        let key = generate_key::<N>()?;
        // 将二进制密钥Base64编码，并返回编码后的字符串
        Ok(base64_encode(&key))
    }

    /// 将字节切片进行Base64编码
    ///
    /// # 参数
    /// * `data`: 待编码的字节切片
    ///
    /// # 返回值
    /// 返回一个Base64编码后的字符串
    pub fn base64_encode(data: &[u8]) -> String {
        general_purpose::STANDARD.encode(data)
    }

    /// Base64解码
    pub fn base64_decode(data: &str) -> Result<Vec<u8>, CryptoError> {
        general_purpose::STANDARD
            .decode(data)
            .map_err(|e| CryptoError::Base64DecodeError(e.to_string()))
    }

    // AES-GCM 模式（原有代码）
    /// 根据密钥长度选择合适的加密算法（GCM模式）
    fn select_cipher(key: &[u8]) -> Result<Cipher, CryptoError> {
        match key.len() {
            AES_128_KEY_LEN => Ok(Cipher::aes_128_gcm()),
            AES_192_KEY_LEN => Ok(Cipher::aes_192_gcm()),
            AES_256_KEY_LEN => Ok(Cipher::aes_256_gcm()),
            len => Err(CryptoError::UnsupportedKeyLength { actual: len }),
        }
    }

    /// 加密选项配置（GCM模式）
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

    /// 使用GCM模式加密
    /// 使用GCM模式加密
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
        // 修正此处的 map_err 调用
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

    /// 使用默认选项加密（GCM模式）
    pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        encrypt_with_options(key, plaintext, &EncryptionOptions::default())
    }

    /// 加密并返回Base64编码结果（GCM模式）
    pub fn encrypt_to_base64(key: &[u8], plaintext: &[u8]) -> Result<String, CryptoError> {
        let ciphertext = encrypt(key, plaintext)?;
        Ok(base64_encode(&ciphertext))
    }

    /// 解密选项配置（GCM模式）
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

    /// 使用GCM模式解密
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

    /// 使用默认选项解密（GCM模式）
    pub fn decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        decrypt_with_options(key, ciphertext, &DecryptionOptions::default())
    }

    /// 解密Base64编码的密文（GCM模式）
    pub fn decrypt_from_base64(
        key: &[u8],
        ciphertext_base64: &str,
    ) -> Result<Vec<u8>, CryptoError> {
        let ciphertext = base64_decode(ciphertext_base64)?;
        decrypt(key, &ciphertext)
    }

    // AES-CBC-192 模式（新增代码）
    /// AES-CBC 加密模式
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AesCbcMode {
        FixedIv,  // 固定IV（全零）
        RandomIv, // 随机IV
    }

    /// 使用AES-CBC-192加密
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
        encrypter.pad(true); // 启用PKCS#7填充

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

    /// 使用AES-CBC-192解密
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

    // 测试代码
    #[cfg(test)]
    mod tests {
        use super::*;

        // GCM模式测试
        #[test]
        fn test_gcm_encryption_decryption() {
            let key = generate_key::<AES_256_KEY_LEN>().unwrap();
            let plaintext = b"Hello, GCM!";

            let ciphertext = encrypt(&key, plaintext).unwrap();
            let decrypted = decrypt(&key, &ciphertext).unwrap();
            assert_eq!(plaintext, decrypted.as_slice());
        }

        // CBC-192 固定IV测试
        #[test]
        fn test_cbc_192_fixed_iv() {
            let key = generate_key::<AES_192_KEY_LEN>().unwrap();
            let plaintext = "Fixed IV Test";

            let ciphertext = encrypt_cbc_192(&key, plaintext, AesCbcMode::FixedIv).unwrap();
            let decrypted = decrypt_cbc_192(&key, &ciphertext).unwrap();
            assert_eq!(plaintext, decrypted);
        }

        // CBC-192 随机IV测试
        #[test]
        fn test_cbc_192_random_iv() {
            let key = generate_key::<AES_192_KEY_LEN>().unwrap();
            let plaintext = "Random IV Test";

            let ciphertext = encrypt_cbc_192(&key, plaintext, AesCbcMode::RandomIv).unwrap();
            let decrypted = decrypt_cbc_192(&key, &ciphertext).unwrap();
            assert_eq!(plaintext, decrypted);
        }

        // 错误处理测试
        #[test]
        fn test_invalid_key_length() {
            let key = "shortkey".as_bytes(); // 10字节
            let plaintext = "Test";

            assert!(encrypt_cbc_192(key, plaintext, AesCbcMode::FixedIv).is_err());
        }
    }
}

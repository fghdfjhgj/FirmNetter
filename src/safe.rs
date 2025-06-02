pub mod safe {
    use ring::{
        aead,
        aead::{Aad, LessSafeKey, Nonce, UnboundKey},
        rand::{SecureRandom, SystemRandom},
    };
    use thiserror::Error;

    // 常量定义
    const AES_128_KEY_LEN: usize = 16;
    const AES_256_KEY_LEN: usize = 32;
    const NONCE_LEN: usize = 12;

    #[derive(Debug, Error)]
    pub enum KeyError {
        #[error("随机密钥生成失败")]
        RandomFailed,
        #[error("密钥长度无效: 必须为 {AES_128_KEY_LEN} 或 {AES_256_KEY_LEN} 字节")]
        InvalidKeyLength,
    }

    /// 生成指定长度的加密密钥。
    ///
    /// 此函数旨在为AES加密算法生成安全的随机密钥。它只支持生成特定长度的密钥，
    /// 即AES_128_KEY_LEN或AES_256_KEY_LEN，以确保加密的安全性。
    ///
    /// # 参数
    /// - `N`: 一个编译时常量，指定所需密钥的长度（以字节为单位）。
    ///
    /// # 返回值
    /// - `Result<[u8; N], KeyError>`: 如果密钥成功生成，返回一个长度为N的字节数组；
    ///   如果生成密钥过程中遇到错误（如不支持的密钥长度或随机数生成失败），则返回相应的错误。
    ///
    /// # 错误处理
    /// - 如果指定的密钥长度N不是AES支持的长度，将返回`KeyError::InvalidKeyLength`错误。
    /// - 如果系统随机数生成器失败，将返回`KeyError::RandomFailed`错误。
    pub fn generate_key<const N: usize>() -> Result<[u8; N], KeyError> {
        // 检查密钥长度是否为支持的AES密钥长度
        if N != AES_128_KEY_LEN && N != AES_256_KEY_LEN {
            return Err(KeyError::InvalidKeyLength);
        }

        // 初始化密钥数组
        let mut key = [0u8; N];
        // 使用系统随机数生成器填充密钥数组
        SystemRandom::new()
            .fill(&mut key)
            .map_err(|_| KeyError::RandomFailed)?;
        // 返回生成的密钥
        Ok(key)
    }

    #[derive(Debug, Error)]
    pub enum CryptoError {
        #[error("加密失败")]
        EncryptionFailed,
        #[error("解密失败")]
        DecryptionFailed,
        #[error("不支持的密钥长度: 需要 {expected} 字节，实际提供 {actual} 字节")]
        UnsupportedKeyLength { expected: usize, actual: usize },
        #[error("密文长度无效")]
        InvalidCiphertextLength,
    }

    /// 根据密钥长度选择合适的加密算法
    ///
    /// # 参数
    ///
    /// * `key` - 一个字节切片，代表加密密钥
    ///
    /// # 返回值
    ///
    /// * `Ok(&'static aead::Algorithm)` - 如果密钥长度匹配支持的算法，返回该算法的静态引用
    /// * `Err(CryptoError)` - 如果密钥长度不支持，则返回一个CryptoError错误
    ///
    /// # 描述
    ///
    /// 该函数根据提供的密钥长度来选择合适的加密算法（目前支持AES-128-GCM和AES-256-GCM）。
    /// 如果密钥长度不匹配任何支持的算法，将返回一个UnsupportedKeyLength错误。
    fn select_algorithm(key: &[u8]) -> Result<&'static aead::Algorithm, CryptoError> {
        match key.len() {
            AES_128_KEY_LEN => Ok(&aead::AES_128_GCM),
            AES_256_KEY_LEN => Ok(&aead::AES_256_GCM),
            len => Err(CryptoError::UnsupportedKeyLength {
                expected: AES_256_KEY_LEN,
                actual: len,
            }),
        }
    }

    /// 使用给定的密钥对明文进行加密。
    ///
    /// # 参数
    ///
    /// - `key`: 用于加密的密钥。
    /// - `plaintext`: 需要加密的明文数据。
    ///
    /// # 返回
    ///
    /// - `Ok(Vec<u8>)`: 加密后的数据，包括nonce、密文和标签。
    /// - `Err(CryptoError)`: 如果加密过程中发生错误，则返回相应的错误。
    pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let algorithm = select_algorithm(key)?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|_| CryptoError::EncryptionFailed)?;

        let unbound_key =
            UnboundKey::new(algorithm, key).map_err(|_| CryptoError::EncryptionFailed)?;
        let less_safe_key = LessSafeKey::new(unbound_key);

        let mut buffer = plaintext.to_vec();
        let tag = less_safe_key
            .seal_in_place_separate_tag(
                // 这里返回的是标签(tag)
                Nonce::assume_unique_for_key(nonce_bytes),
                Aad::empty(),
                &mut buffer,
            )
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // 正确结构：nonce(12) + 密文 + tag(16)
        let mut result = nonce_bytes.to_vec();
        result.extend(buffer); // 密文
        result.extend(tag.as_ref()); // 添加标签
        Ok(result)
    }

    /// 解密给定的密文。
    ///
    /// # 参数
    ///
    /// - `key`: 解密密文的密钥。
    /// - `ciphertext`: 要解密的密文。
    ///
    /// # 返回
    ///
    /// - `Ok(Vec<u8>)`: 解密后的数据。
    /// - `Err(CryptoError)`: 如果解密失败，则返回错误。
    pub fn decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let algorithm = select_algorithm(key)?;
        let tag_len = algorithm.tag_len();

        if ciphertext.len() < NONCE_LEN + tag_len {
            return Err(CryptoError::InvalidCiphertextLength);
        }

        let nonce = Nonce::try_assume_unique_for_key(&ciphertext[..NONCE_LEN])
            .map_err(|_| CryptoError::DecryptionFailed)?;

        let unbound_key =
            UnboundKey::new(algorithm, key).map_err(|_| CryptoError::DecryptionFailed)?;
        let less_safe_key = LessSafeKey::new(unbound_key);

        // 创建包含密文+标签的缓冲区
        let mut buffer = ciphertext[NONCE_LEN..].to_vec();

        // 解密并获取明文长度
        let plaintext_len = less_safe_key
            .open_in_place(nonce, Aad::empty(), &mut buffer)
            .map_err(|_| CryptoError::DecryptionFailed)?
            .len();

        // 只返回实际明文部分
        Ok(buffer[..plaintext_len].to_vec())
    }
    #[test]
    fn test() {
        let key = generate_key::<32>().expect("generate key failed");
        println!("key: {:?}", key);
        let text = "herld";
        println!("text: {}", text);
        let ciphertext = encrypt(&key, text.as_ref()).expect("encrypt failed");
        println!("ciphertext: {:?}", ciphertext);
        let plaintext = decrypt(&key, &ciphertext).expect("decrypt failed");
        println!("plaintext: {}", String::from_utf8_lossy(&plaintext));
    }
}

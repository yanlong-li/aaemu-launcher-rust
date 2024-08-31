use std::str;
use rc4::{KeyInit, Rc4, StreamCipher};

// 使用 AES-128-CBC 解密
pub fn decrypt(plain_data: &[u8], encryption_key: &[u8; 8]) -> Result<Vec<u8>, &'static str> {
    let mut rc4 = Rc4::new(encryption_key.into());

    // 创建 `plain_data` 的一个可变副本
    let mut data = plain_data.to_vec(); // 将不可变切片转换为可变的 Vec<u8>

    rc4.apply_keystream(&mut data);
    Ok(data)
}

// 使用 AES-128-CBC 解密
// pub fn encrypt(plain_data: &mut [u8], iv: &[u8]) -> Result<String, &'static str> {
//
//
//     println!("Ciphertext (hex): {:?}", ciphertext);
//
//     // Ok("output".to_string())
//     // 将加密数据转换为 Base64 编码的字符串以便于展示
//     let encrypted_string = general_purpose::STANDARD.encode(&ciphertext);
//     Ok(encrypted_string)
// }

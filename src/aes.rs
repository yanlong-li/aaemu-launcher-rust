use openssl::symm::{Cipher, Crypter, Mode};
use std::str;
use base64::Engine;
use base64::engine::general_purpose;
use openssl::rand::rand_bytes;

// 设置使用的 AES 密钥和初始化向量（IV）
const KEY: &[u8; 16] = b"aaaaaaaaaaaaaaaa"; // AES-128 密钥
const IV: &[u8; 32] = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"; // AES-128 密钥

// 使用 AES-128-CBC 解密
pub fn decrypt_aes128_cbc(plain_data: &[u8], iv: &[u8]) -> Result<String, &'static str> {
    let ciphertext = general_purpose::STANDARD.decode(&plain_data).expect("解密失败");

    Ok(String::from_utf8(ciphertext.to_vec()).expect("转换失败"))

    // let mut iv = [0u8; 16];
    // rand_bytes(&mut iv).expect("Failed to generate random IV");
    //
    // let cipher = Cipher::aes_128_cbc();
    //
    // // 解密
    // let mut crypter = Crypter::new(cipher, Mode::Decrypt, KEY, Some(&iv))
    //     .expect("Failed to create Crypter");
    // let mut decrypted_data = ciphertext.clone();
    // let count = crypter.update(&ciphertext, &mut decrypted_data)
    //     .expect("Failed to decrypt");
    // let rest = crypter.finalize(&mut decrypted_data[count..])
    //     .expect("Failed to finalize decryption");
    // decrypted_data.truncate(count + rest);
    //
    //
    // Ok("123".parse().unwrap())
}

// 使用 AES-128-CBC 解密
pub fn encrypt_aes128_cbc(plain_data: &[u8], iv: &[u8]) -> Result<String, &'static str> {
    let mut iv = [0u8; 16];
    rand_bytes(&mut iv).expect("Failed to generate random IV");


    let cipher = Cipher::aes_128_cbc();

    // 要加密的数据

    // 加密
    let mut crypter = Crypter::new(cipher, Mode::Encrypt, KEY, Some(&iv))
        .expect("Failed to create Crypter");
    let mut ciphertext = vec![0; plain_data.len() + cipher.block_size()];
    let count = crypter.update(plain_data, &mut ciphertext)
        .expect("Failed to encrypt");
    let rest = crypter.finalize(&mut ciphertext[count..])
        .expect("Failed to finalize encryption");
    ciphertext.truncate(count + rest);

    println!("Ciphertext (hex): {:?}", ciphertext);

    // Ok("output".to_string())
    // 将加密数据转换为 Base64 编码的字符串以便于展示
    let encrypted_string = general_purpose::STANDARD.encode(&ciphertext);
    Ok(encrypted_string)
}

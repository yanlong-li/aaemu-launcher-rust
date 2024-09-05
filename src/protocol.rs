use std::env::args;

use base64::Engine;
use base64::engine::general_purpose;
use rand::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AuthToken {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "p")]
    pub password: String,
    #[serde(rename = "s")]
    pub server: String,
    #[serde(rename = "P")]
    pub port: u16,
    #[serde(rename = "v")]
    pub launcher_version: u16,
    #[serde(rename = "mv")]
    pub with_launcher_version: u16,
    #[serde(rename = "dh")]
    pub db_hash: String,
}


pub fn handle() -> Result<AuthToken, Box<dyn std::error::Error>> {
    let args = args();

    if args.len() < 2 {
        return Err(Error::new("没有协议参数").into());
    }
    let args_vec: Vec<_> = args.into_iter().collect();
    let schema = args_vec.get(1);

    match schema {
        None => {
            Err(Error::new("协议参数无效").into())
        }
        Some(url) => {
            if !url.starts_with("plaa://") {
                return Err(Error::new("协议参数无效").into());
            }

            let b64_data = url.get(7..).expect("数据不完整");
            let data = general_purpose::STANDARD.decode(&b64_data).expect("无法解码数据");

            let iv = data.get(0..8).expect("获取IV失败");
            let ciphertext = data.get(8..).expect("获取数据失败");

            println!("协议内容 {}", url);
            let plaintext = super::cipher::decrypt(ciphertext, <&[u8; 8]>::try_from(iv).unwrap())?;
            let auth_token: AuthToken = serde_json::from_slice(plaintext.as_slice()).expect("数据损坏");
            Ok(auth_token)
        }
    }
}
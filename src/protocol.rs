use std::env::args;

use base64::Engine;
use base64::engine::general_purpose;
use rand::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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


pub async fn handle() -> Result<AuthToken, Box<dyn std::error::Error>> {
    let args = args();

    if args.len() < 2 {
        return Err(Error::new("请在官网点击“开始游戏”按钮。").into());
    }
    let args_vec: Vec<_> = args.into_iter().collect();
    let schema = args_vec.get(1);

    match schema {
        None => {
            Err(Error::new("协议参数无效").into())
        }
        Some(url) => {
            let mut b64_data = url.get(7..).ok_or_else(|| Error::new("数据不完整"))?;

            // 如果存在 '&'，则截取 '&' 之前的内容
            if let Some(pos) = b64_data.find('&') {
                b64_data = &b64_data[..pos];
            }

            let data = general_purpose::STANDARD
                .decode(b64_data)
                .map_err(|_| Error::new("无法解码数据,请联系管理员！"))?;

            let iv = data.get(0..8).ok_or_else(|| Error::new("令牌解密失败，获取IV失败！"))?;
            let ciphertext = data.get(8..).ok_or_else(|| Error::new("令牌解密失败，获取数据失败！"))?;

            println!("协议内容 {}", url);
            let plaintext = super::cipher::decrypt(ciphertext, <&[u8; 8]>::try_from(iv).unwrap())?;
            let auth_token: AuthToken = serde_json::from_slice(plaintext.as_slice()).expect("令牌解密失败，数据损坏！");
            Ok(auth_token)
        }
    }
}
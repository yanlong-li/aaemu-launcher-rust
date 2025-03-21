use std::env::args;

use base64::engine::general_purpose;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

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
    #[serde(rename = "d")]
    #[serde(default = "domain")]
    pub domain: String,
}

pub fn domain() -> String {
    String::from("https://plaa.top")
}

#[derive(Debug)]
struct MyError {
    message: String,
}

impl MyError {
    fn new(message: &str) -> Self {
        MyError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "令牌错误: {}", self.message)
    }
}

impl Error for MyError {}

pub async fn handle() -> Result<AuthToken, Box<dyn Error>> {
    let args = args();

    if args.len() < 2 {
        return Err(MyError::new("请在官网点击“开始游戏”按钮。").into());
    }
    let args_vec: Vec<_> = args.into_iter().collect();
    let schema = args_vec.get(1);

    match schema {
        None => Err(MyError::new("协议参数无效").into()),
        Some(url) => {
            let mut b64_data = url.get(7..).expect("协议长度错误");

            // 如果存在 '&'，则截取 '&' 之前的内容
            if let Some(pos) = b64_data.find('&') {
                b64_data = &b64_data[..pos];
            }

            let data = general_purpose::STANDARD
                .decode(b64_data)
                .expect("Base64解码失败");

            let iv = data.get(0..8).expect("IV获取失败");
            let ciphertext = data.get(8..).expect("密文获取失败");

            println!("协议内容 {}", url);
            let plaintext = super::cipher::decrypt(ciphertext, <&[u8; 8]>::try_from(iv).unwrap())?;
            let auth_token: AuthToken = serde_json::from_slice(plaintext.as_slice())
                .or_else(|_| Err(MyError::new("序列化失败")))?;
            Ok(auth_token)
        }
    }
}

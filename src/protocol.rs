use rand::Error;
use std::env::args;
use std::thread::sleep;
use std::time::Duration;

struct Config {
    pub username: String,
    pub password: String,
    pub db_key: String,
    pub server: String,
    pub port: u16,
}


pub fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let args = args();

    if (args.len() < 2) {
        return Err(Error::new("没有协议参数").into());
    }
    let args_vec: Vec<_> = args.into_iter().collect();
    let schema = args_vec.get(1);

    match schema {
        None => {
            Err(Error::new("协议参数无效").into())
        }
        Some(url) => {
            if (!url.starts_with("plaa://")) {
                return Err(Error::new("协议参数无效").into());
            }

            ;
            let s = url.as_str();

            // let iv = s.get(7..7 + 16).expect("获取IV失败");
            //
            // println!("协议内容 {}", url);
            // println!("iv {}", iv);

            let data = "你好啊".as_bytes();
            let plaintext = b"aaaaaaaaaaaaaaaa";
            let res = super::aes::encrypt_aes128_cbc(plaintext)?;
            let data2 = res.as_bytes();
            let r2 = super::aes::decrypt_aes128_cbc(data2)?;

            println!("{}",res);
            println!("{}",r2);


            sleep(Duration::from_secs(3));
            Ok(())
        }
    }
}
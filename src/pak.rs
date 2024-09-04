use std::fs::{File, Metadata};
use std::io;
use std::io::BufRead;

mod aa_pak_file_info;
mod aa_pak;
mod aa_pak_file_format_reader;
mod aa_pak_file_header;
mod aa_pak_loading_progress_type;
mod packer_sub_stream;
mod pak_file_type;

struct PakFileInfo {
    file_path: String,
    file_size: i64,
    file_hash: String,
    file_info: Metadata,
}

pub fn handle() {
    let list: Vec<PakFileInfo> = vec![];

    let ignore_list = read_ignore_file_list();

    println!("{:?}", ignore_list)
}

fn read_ignore_file_list() -> Vec<String> {
    let mut ignore_list = vec![];
    ignore_list.push(String::from("ignore.txt"));

    // 尝试打开 ignore.txt 文件
    if let Ok(file) = File::open("ignore.txt") {
        // 使用 BufReader 按行读取文件
        let reader = io::BufReader::new(file);

        // 将每一行的内容添加到 ignore_list
        for line in reader.lines() {
            match line {
                Ok(content) => ignore_list.push(content),
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }
    } else {
        eprintln!("Failed to open ignore.txt");
    }

    ignore_list
}
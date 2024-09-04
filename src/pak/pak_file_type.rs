/// Pak 文件类型枚举
#[derive(Debug)]
pub enum PakFileType {
    /// FormatReader 类型的 pak 文件
    Reader,
    /// 经典的“普通” pak 文件
    Classic,
    /// 从 CSV 数据创建的虚拟 pak 文件
    Csv,
}
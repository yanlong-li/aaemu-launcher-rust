use std::time::SystemTime;

/// File Details Block
#[derive(Debug, Clone)]
pub struct AAPakFileInfo {
    /// Original file creation time
    pub create_time: i64,

    /// Index of this deleted file
    pub deleted_index_number: i32,

    /// Unknown value 1, mostly 0 or 0x80000000 observed, possible file flags ?
    pub dummy1: u32,

    /// Unknown value 2, observed to be 0, seems to be unused
    pub dummy2: u64,

    /// Index in the normal files list
    pub entry_index_number: i32,

    /// MD5 Hash byte array (should be 16 bytes)
    pub md5: [u8; 16],

    /// Original file modified time
    pub modify_time: i64,

    /// Filename inside of the pakFile
    pub name: String,

    /// Offset in bytes of the starting location inside the pakFile
    pub offset: i64,

    /// Number of bytes of free space left until the next blockSize of 512 (or space until next file)
    pub padding_size: i64,

    /// Original fileSize
    pub size: i64,

    /// Duplicate of the original fileSize? Possibly file after decompression?
    /// Always observed as being the same as fileSize
    pub size_duplicate: i64,
}

impl AAPakFileInfo {
    pub fn new(
        create_time: i64,
        deleted_index_number: i32,
        dummy1: u32,
        dummy2: u64,
        entry_index_number: i32,
        md5: [u8; 16],
        modify_time: i64,
        name: String,
        offset: i64,
        padding_size: i64,
        size: i64,
        size_duplicate: i64,
    ) -> Self {
        AAPakFileInfo {
            create_time,
            deleted_index_number,
            dummy1,
            dummy2,
            entry_index_number,
            md5,
            modify_time,
            name,
            offset,
            padding_size,
            size,
            size_duplicate,
        }
    }

    pub fn default() -> AAPakFileInfo {
        Self{
            create_time: 0,
            deleted_index_number: 0,
            dummy1: 0,
            dummy2: 0,
            entry_index_number: 0,
            md5: [0;16],
            modify_time: 0,
            name: "".to_string(),
            offset: 0,
            padding_size: 0,
            size: 0,
            size_duplicate: 0,
        }
    }
}
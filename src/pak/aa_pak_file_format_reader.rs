use std::collections::VecDeque;

/// Possible elements for Pak Header
#[derive(Debug, Clone, Copy)]
pub enum AAPakFileHeaderElement {
    AnyByte,
    NullByte,
    Header,
    FilesCount,
    ExtraFilesCount,
}

/// Possible elements for File Meta data
#[derive(Debug, Clone, Copy)]
pub enum AAPakFileInfoElement {
    FileName,
    Offset,
    Size,
    SizeDuplicate,
    PaddingSize,
    Md5,
    Dummy1,
    CreateTime,
    ModifyTime,
    Dummy2,
}

/// Reader class defining how file meta data should be read and written for AAPak
#[derive(Debug, Clone)]
pub struct AAPakFileFormatReader {
    /// Name of this reader
    pub reader_name: String,

    /// Marks if this Reader has been created with initializeWithDefaults enabled
    pub is_default: bool,

    /// Encryption Key to use for header data
    pub header_encryption_key: Vec<u8>,

    /// Header identification bytes (4)
    pub header_bytes: Vec<u8>,

    /// Read order of elements for the header
    pub read_order: VecDeque<AAPakFileHeaderElement>,

    /// Set to true if the FAT stores Extra Files before Normal Files
    pub invert_file_counter: bool,

    /// Read order for File Info in FAT entry
    pub file_info_read_order: VecDeque<AAPakFileInfoElement>,

    /// Default values to use for Dummy1 on new entries
    pub default_dummy1: u32,

    /// Default values to use for Dummy2 on new entries
    pub default_dummy2: u32,
}

impl AAPakFileFormatReader {
    /// Creates a format reader object
    pub fn new(initialize_with_defaults: bool) -> Self {
        let mut reader = AAPakFileFormatReader {
            reader_name: String::from("None"),
            is_default: false,
            header_encryption_key: Vec::new(),
            header_bytes: Vec::new(),
            read_order: VecDeque::new(),
            invert_file_counter: false,
            file_info_read_order: VecDeque::new(),
            default_dummy1: 0,
            default_dummy2: 0,
        };

        if initialize_with_defaults {
            reader.reader_name = String::from("Default");
            reader.is_default = true;
            reader.header_encryption_key = AAPakFileFormatReader::xl_games_key().to_vec();
            reader.header_bytes = vec![0x57, 0x49, 0x42, 0x4F];
            reader.read_order = vec![
                AAPakFileHeaderElement::Header,
                AAPakFileHeaderElement::NullByte,
                AAPakFileHeaderElement::NullByte,
                AAPakFileHeaderElement::NullByte,
                AAPakFileHeaderElement::NullByte,
                AAPakFileHeaderElement::FilesCount,
                AAPakFileHeaderElement::ExtraFilesCount,
                AAPakFileHeaderElement::NullByte,
                AAPakFileHeaderElement::NullByte,
                AAPakFileHeaderElement::NullByte,
                AAPakFileHeaderElement::NullByte,
            ].into();
            reader.invert_file_counter = false;
            reader.file_info_read_order = vec![
                AAPakFileInfoElement::FileName,
                AAPakFileInfoElement::Offset,
                AAPakFileInfoElement::Size,
                AAPakFileInfoElement::SizeDuplicate,
                AAPakFileInfoElement::PaddingSize,
                AAPakFileInfoElement::Md5,
                AAPakFileInfoElement::Dummy1,
                AAPakFileInfoElement::CreateTime,
                AAPakFileInfoElement::ModifyTime,
                AAPakFileInfoElement::Dummy2,
            ].into();
        }

        reader
    }

    /// Default AES128 key used by XLGames for ArcheAge as encryption key for header and fileInfo data
    pub fn xl_games_key() -> &'static [u8] {
        &[0x32, 0x1F, 0x2A, 0xEE, 0xAA, 0x58, 0x4A, 0xB4, 0x9A, 0x6C, 0x9E, 0x09, 0xD5, 0x9E, 0x9C, 0x6F]
    }
}

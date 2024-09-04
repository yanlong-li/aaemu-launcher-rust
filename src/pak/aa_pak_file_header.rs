use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{Cursor, SeekFrom};
use std::io::prelude::*;
use aes::{Aes128, BLOCK_SIZE};
use block_modes::{BlockMode, Cbc};
use block_padding::NoPadding;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::pak::aa_pak::AAPak;
use crate::pak::aa_pak_file_format_reader::{AAPakFileFormatReader, AAPakFileHeaderElement, AAPakFileInfoElement};
use crate::pak::aa_pak_file_info::AAPakFileInfo;
use crate::pak::aa_pak_loading_progress_type::AAPakLoadingProgressType;
use crate::pak::pak_file_type::PakFileType;

/// File Header Size Constants
const HEADER_SIZE: usize = 0x200;
const FILE_INFO_SIZE: usize = 0x150;

/// Default AES128 key used by XLGames for ArcheAge
const XLGAMES_KEY: [u8; 16] = [
    0x32, 0x1F, 0x2A, 0xEE, 0xAA, 0x58, 0x4A, 0xB4,
    0x9A, 0x6C, 0x9E, 0x09, 0xD5, 0x9E, 0x9C, 0x6F,
];

type Aes128Cbc = Cbc<Aes128, NoPadding>;

/// File Header Structure
pub struct AAPakFileHeader {
    /// Empty MD5 Hash to compare against
    pub null_hash: [u8; 16],

    /// Empty MD5 Hash as a hex string to compare against
    pub null_hash_string: String,

    /// Exception error message from the last thrown encoding/decoding error
    pub last_aes_error: String,

    /// Default AES128 key used by XLGames for ArcheAge as encryption key for header and fileInfo data
    xl_games_key: [u8; 16],

    /// Reference to owning pakFile object
    owner: Option<AAPak>,

    /// Offset in pakFile where to start adding new files
    add_file_offset: i64,

    /// Decrypted Header data
    data: [u8; HEADER_SIZE],

    /// Number of unused "deleted" files inside this pak
    extra_file_count: u32,

    /// Memory stream that holds the encrypted file information + header part of the file
    pub(crate) fat: Cursor<Vec<u8>>,

    /// Number of used files inside this pak
    file_count: u32,

    /// Offset in pakFile where the meta data of the first file in the list is stored
    pub first_file_info_offset: i64,

    /// Is this header valid
    pub is_valid: bool,

    /// Current encryption key
    key: [u8; 16],

    /// Unencrypted header
    pub raw_data: [u8; HEADER_SIZE],

    /// Header Size
    pub size: usize,
}

impl AAPakFileHeader {
    /// Creates a new Header Block for a Pak file
    pub fn new(owner: Option<AAPak>) -> Self {
        let mut header = AAPakFileHeader {
            null_hash: [0; 16],
            null_hash_string: "00000000000000000000000000000000".to_string(),
            last_aes_error: String::new(),
            xl_games_key: XLGAMES_KEY,
            owner,
            add_file_offset: 0,
            data: [0; HEADER_SIZE],
            extra_file_count: 0,
            fat: Cursor::new(vec![]),
            file_count: 0,
            first_file_info_offset: 0,
            is_valid: false,
            key: XLGAMES_KEY,
            raw_data: [0; HEADER_SIZE],
            size: HEADER_SIZE,
        };
        header.set_default_key();
        header
    }

    /// If you want to use custom keys on your pak file, use this function to change the key that is used for
    /// encryption/decryption of the FAT and header data
    pub fn set_custom_key(&mut self, new_key: [u8; 16]) {
        self.key = new_key;
    }

    /// Reverts back to the original XL encryption key
    pub fn set_default_key(&mut self) {
        self.key = XLGAMES_KEY;
    }


    /// Encrypts or decrypts data using AES with CBC mode.
    pub fn encrypt_aes(message: &[u8], key: &[u8], do_encryption: bool) -> Result<Vec<u8>, Box<dyn Error>> {
        let cipher = Aes128Cbc::new_from_slices(key, &[0; 16])?;

        let mut output = vec![0u8; message.len()];
        if do_encryption {
            cipher.encrypt(&mut output, message.len())?;
        } else {
            cipher.decrypt(&mut output)?;
        }

        Ok(output)
    }

    /// Encrypts or decrypts data from a source stream to a target stream using AES with CBC mode.
    pub fn encrypt_stream_aes<R: Read, W: Write>(
        source: &mut R,
        target: &mut W,
        key: &[u8],
        do_encryption: bool,
        leave_open: bool,
    ) -> Result<bool, Box<dyn Error>> {
        let cipher = Aes128Cbc::new_from_slices(key, &[0; 16])?;

        // Create a buffer to read from the source stream and write to the target stream
        let mut buffer = [0u8; 4096];
        let mut output_buffer = Vec::new();

        // Encrypt or decrypt the data
        while let Ok(bytes_read) = source.read(&mut buffer) {
            if bytes_read == 0 {
                break;
            }

            if do_encryption {
                let mut encryptor = cipher.encryptor();
                let result = encryptor.encrypt(&buffer[..bytes_read], &mut output_buffer)?;
                target.write_all(&result)?;
            } else {
                let mut decryptor = cipher.decryptor();
                let result = decryptor.decrypt(&buffer[..bytes_read], &mut output_buffer)?;
                target.write_all(&result)?;
            }
        }

        if !leave_open {
            target.flush()?;
        }

        Ok(true)
    }

    /// Load the encrypted FAT data into memory.
    pub fn load_raw_fat(&mut self) -> Result<bool, Box<dyn Error>> {
        // Trigger progress update (Placeholder function)
        self.trigger_progress(AAPakLoadingProgressType::ReadingFAT, 0, 100);

        // Calculate total file info size
        let total_file_info_size = (self.file_count + self.extra_file_count) as usize * FILE_INFO_SIZE;

        let mut gp_file_stream = self.owner.take().unwrap().gp_file_stream.unwrap();
        // Move to the end of the file
        gp_file_stream.seek(SeekFrom::End(0))?;
        self.first_file_info_offset = gp_file_stream.stream_position()? as i64;

        // Align to previous block of 512 bytes
        self.first_file_info_offset -= HEADER_SIZE as u64;
        self.first_file_info_offset -= total_file_info_size as u64;
        let dif = (self.first_file_info_offset as usize % BLOCK_SIZE) as usize;
        self.first_file_info_offset -= dif as u64;

        // Set the file position to the calculated offset
        gp_file_stream.seek(SeekFrom::Start(self.first_file_info_offset as u64))?;

        // Read the FAT data
        let mut fat_data = Vec::new();
        gp_file_stream.read_to_end(&mut fat_data)?;
        self.fat = Cursor::new(fat_data);

        // Trigger progress update (Placeholder function)
        self.trigger_progress(AAPakLoadingProgressType::ReadingFAT, 100, 100);

        Ok(true)
    }

    /// Placeholder function for triggering progress updates
    fn trigger_progress(&self, progress_type: AAPakLoadingProgressType, step: u32, maximum: u32) {
        // Implement the progress triggering logic here
        println!("{:?} - {}/{}", progress_type, step, maximum);
    }

    pub fn write_to_fat(&mut self) -> Result<bool, Box<dyn Error>> {
        if self.pak_type == PakFileType::Csv {
            return Ok(false);
        }

        self.trigger_progress(AAPakLoadingProgressType::WritingFAT, 0, 100);

        // Prepare buffer and stream for writing
        self.fat.clear();
        let buf_size = FILE_INFO_SIZE;
        let mut ms = Vec::with_capacity(buf_size);
        let mut writer = Vec::with_capacity(buf_size);

        let total_file_count = self.files.len() as u32 + self.extra_files.len() as u32;
        let mut files_to_go = self.files.len() as u32;
        let mut extras_to_go = self.extra_files.len() as u32;
        let mut file_index = 0;
        let mut extras_index = 0;

        self.trigger_progress(AAPakLoadingProgressType::WritingFAT, 0, total_file_count);

        let inverted_order = matches!(self.pak_type, PakFileType::Reader)
            && self.reader.as_ref().map_or(false, |r| r.invert_file_counter);

        for i in 0..total_file_count {
            ms.clear();
            let pfi = if !inverted_order {
                if files_to_go > 0 {
                    files_to_go -= 1;
                    self.files[file_index].clone()
                } else if extras_to_go > 0 {
                    extras_to_go -= 1;
                    self.extra_files[extras_index].clone()
                } else {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "PC cannot math!",
                    )));
                }
            } else {
                if extras_to_go > 0 {
                    extras_to_go -= 1;
                    self.extra_files[extras_index].clone()
                } else if files_to_go > 0 {
                    files_to_go -= 1;
                    self.files[file_index].clone()
                } else {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "PC cannot math!",
                    )));
                }
            };

            // Write file info based on the format
            if matches!(self.pak_type, PakFileType::Reader) {
                if let Some(ref reader) = self.reader {
                    for file_info_read_order in &reader.file_info_read_order {
                        match file_info_read_order {
                            AAPakFileInfoElement::FileName => {
                                writer.extend_from_slice(&pfi.name.as_bytes());
                                writer.resize(0x108, 0);
                            }
                            AAPakFileInfoElement::Offset => writer.extend_from_slice(&pfi.offset.to_le_bytes()),
                            AAPakFileInfoElement::Size => writer.extend_from_slice(&pfi.size.to_le_bytes()),
                            AAPakFileInfoElement::SizeDuplicate => writer.extend_from_slice(&pfi.size_duplicate.to_le_bytes()),
                            AAPakFileInfoElement::PaddingSize => writer.extend_from_slice(&pfi.padding_size.to_le_bytes()),
                            AAPakFileInfoElement::Md5 => writer.extend_from_slice(&pfi.md5),
                            AAPakFileInfoElement::CreateTime => writer.extend_from_slice(&pfi.create_time.to_le_bytes()),
                            AAPakFileInfoElement::ModifyTime => writer.extend_from_slice(&pfi.modify_time.to_le_bytes()),
                            AAPakFileInfoElement::Dummy1 => writer.extend_from_slice(&pfi.dummy1.to_le_bytes()),
                            AAPakFileInfoElement::Dummy2 => writer.extend_from_slice(&pfi.dummy2.to_le_bytes()),
                            _ => return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Invalid FileInfoElement",
                            ))),
                        }
                    }
                }
            } else if matches!(self.pak_type, PakFileType::Classic) {
                writer.extend_from_slice(&pfi.name.as_bytes());
                writer.resize(0x108, 0);
                writer.extend_from_slice(&pfi.offset.to_le_bytes());
                writer.extend_from_slice(&pfi.size.to_le_bytes());
                writer.extend_from_slice(&pfi.size_duplicate.to_le_bytes());
                writer.extend_from_slice(&pfi.padding_size.to_le_bytes());
                writer.extend_from_slice(&pfi.md5);
                writer.extend_from_slice(&pfi.dummy1.to_le_bytes());
                writer.extend_from_slice(&pfi.create_time.to_le_bytes());
                writer.extend_from_slice(&pfi.modify_time.to_le_bytes());
                writer.extend_from_slice(&pfi.dummy2.to_le_bytes());
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unknown file format",
                )));
            }

            let raw_file_data = Self::encrypt_aes(&writer, &self.key, true)?;
            self.fat.extend_from_slice(&raw_file_data);

            if i % self.on_progress_fat_file_interval == 0 {
                self.trigger_progress(AAPakLoadingProgressType::WritingFAT, i, total_file_count);
            }
        }

        // Add padding
        let dif = self.fat.len() % BLOCK_SIZE;
        if dif > 0 {
            let pad = BLOCK_SIZE - dif;
            self.fat.resize(self.fat.len() + pad, 0);
        }

        // Update header info
        self.file_count = self.files.len() as u32;
        self.extra_file_count = self.extra_files.len() as u32;
        self.fat.resize(self.fat.len() + HEADER_SIZE, 0);
        self.encrypt_header_data()?;
        self.fat[0..0x20].copy_from_slice(&self.raw_data[0..0x20]);

        self.trigger_progress(AAPakLoadingProgressType::WritingFAT, total_file_count, total_file_count);
        Ok(true)
    }

    /// <summary>
    /// Read and decrypt the File Details Table that was loaded into the FAT MemoryStream
    /// </summary>
    pub(crate) fn read_file_table(&mut self) {
        if self.owner == None {
            return;
        }

        let mut owner = &mut self.owner;
        owner.trigger_progress(AAPakLoadingProgressType::ReadingFAT, 0, 100);

        const BUF_SIZE: usize = 0x150; // Marshal.SizeOf(typeof(AAPakFileInfo));

        // Check aa.bms QuickBMS file for reference
        self.fat.set_position(0);
        let mut ms = Vec::with_capacity(BUF_SIZE);
        let mut reader = Cursor::new(&mut ms);

        // Read the Files
        owner.files.clear();
        owner.extra_files.clear();
        let total_file_count = self.file_count + self.extra_file_count;
        let mut files_to_go = self.file_count;
        let mut extra_to_go = self.extra_file_count;
        let mut file_index_counter = -1;
        let mut deleted_index_counter = -1;
        owner.trigger_progress(AAPakLoadingProgressType::ReadingFAT, 0, total_file_count as i32);

        let inverted_order = matches!(owner.pak_type, PakFileType::Reader) && owner.reader.is_some() && owner.reader.as_ref().unwrap().invert_file_counter;

        for i in 0..total_file_count {
            // Read and decrypt a fileInfo block
            let mut raw_file_data = vec![0u8; BUF_SIZE];
            self.fat.read_exact(&mut raw_file_data).unwrap();
            let decrypted_file_data = self.encrypt_aes(&raw_file_data, &self.key, false);

            // Read decrypted data into a AAPakFileInfo
            ms.clear();
            ms.extend_from_slice(&decrypted_file_data);
            let mut pfi = AAPakFileInfo::default();

            reader.set_position(0);

            if matches!(owner.pak_type, PakFileType::Reader) && owner.reader.is_some() {
                for file_info_read_order in &owner.reader.as_ref().unwrap().file_info_read_order {
                    match file_info_read_order {
                        AAPakFileInfoElement::FileName => {
                            pfi.name.clear();
                            let start_pos = reader.position();
                            for _ in 0..0x108 {
                                let ch = reader.read_u8().unwrap();
                                if ch != 0 {
                                    pfi.name.push(ch as char);
                                } else {
                                    break;
                                }
                            }
                            reader.set_position(start_pos + 0x108);
                        }
                        AAPakFileInfoElement::Offset => pfi.offset = reader.read_i64().unwrap(),
                        AAPakFileInfoElement::Size => pfi.size = reader.read_i64().unwrap(),
                        AAPakFileInfoElement::SizeDuplicate => pfi.size_duplicate = reader.read_i64().unwrap(),
                        AAPakFileInfoElement::PaddingSize => pfi.padding_size = reader.read_i32().unwrap(),
                        AAPakFileInfoElement::Md5 => pfi.md5.copy_from_slice(&reader.read_bytes(16)),
                        AAPakFileInfoElement::Dummy1 => pfi.dummy1 = reader.read_u32().unwrap(),
                        AAPakFileInfoElement::CreateTime => pfi.create_time = reader.read_i64().unwrap(),
                        AAPakFileInfoElement::ModifyTime => pfi.modify_time = reader.read_i64().unwrap(),
                        AAPakFileInfoElement::Dummy2 => pfi.dummy2 = reader.read_u64().unwrap(),
                        _ => panic!("Unsupported file info element"),
                    }
                }
            } else if matches!(owner.pak_type, PakFileType::Classic) {
                pfi.name.clear();
                for _ in 0..0x108 {
                    let ch = reader.read_u8().unwrap();
                    if ch != 0 {
                        pfi.name.push(ch as char);
                    } else {
                        break;
                    }
                }
                reader.set_position(0x108);
                pfi.offset = reader.read_i64().unwrap();
                pfi.size = reader.read_i64().unwrap();
                pfi.size_duplicate = reader.read_i64().unwrap();
                pfi.padding_size = reader.read_i32().unwrap();
                pfi.md5.copy_from_slice(&reader.read_bytes(16));
                pfi.dummy1 = reader.read_u32().unwrap();
                pfi.create_time = reader.read_i64().unwrap();
                pfi.modify_time = reader.read_i64().unwrap();
                pfi.dummy2 = reader.read_u64().unwrap();
            } else {
                panic!("Unsupported file type");
            }

            // Handle order file counting
            if !inverted_order {
                if files_to_go > 0 {
                    file_index_counter += 1;
                    pfi.entry_index_number = file_index_counter;

                    files_to_go -= 1;
                    owner.files.push(pfi);
                } else if extra_to_go > 0 {
                    deleted_index_counter += 1;
                    pfi.deleted_index_number = deleted_index_counter;

                    extra_to_go -= 1;
                    owner.extra_files.push(pfi);
                }
            } else {
                if extra_to_go > 0 {
                    file_index_counter += 1;
                    pfi.entry_index_number = file_index_counter;

                    extra_to_go -= 1;
                    owner.extra_files.push(pfi);
                } else if files_to_go > 0 {
                    deleted_index_counter += 1;
                    pfi.deleted_index_number = deleted_index_counter;

                    files_to_go -= 1;
                    owner.files.push(pfi);
                }
            }

            // determine the newest date, use both creating and modify date for this
            if let Ok(f_time) = DateTime::from_file_time(pfi.create_time) {
                if f_time > owner.newest_file_date {
                    owner.newest_file_date = f_time;
                }
            }

            if let Ok(f_time) = DateTime::from_file_time(pfi.modify_time) {
                if f_time > owner.newest_file_date {
                    owner.newest_file_date = f_time;
                }
            }

            // Update our "end of file data" location if needed
            if pfi.offset + pfi.size + pfi.padding_size > self.add_file_offset {
                self.add_file_offset = pfi.offset + pfi.size + pfi.padding_size;
            }

            if i % owner.on_progress_fat_file_interval == 0 {
                owner.trigger_progress(AAPakLoadingProgressType::ReadingFAT, i as i32, total_file_count as i32);
            }
        }

        owner.trigger_progress(AAPakLoadingProgressType::ReadingFAT, total_file_count as i32, total_file_count as i32);
    }


    /// Converts a byte array to a hex string representation.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The input byte array.
    /// * `spacing_text` - String to use for spacing between bytes (default is a space).
    /// * `line_feed` - String to use as newline every 16 bytes (default is "\r\n").
    ///
    /// # Returns
    ///
    /// A string representing the byte array in hexadecimal format.
    fn byte_array_to_hex_string(
        bytes: &[u8],
        spacing_text: &str,
        line_feed: &str,
    ) -> String {
        let mut s = String::new();
        for (i, &byte) in bytes.iter().enumerate() {
            s.push_str(&format!("{:02X}{}", byte, spacing_text));
            if i % 16 == 15 {
                s.push_str(line_feed);
            } else {
                if i % 4 == 3 {
                    s.push_str(spacing_text);
                }
                if i % 8 == 7 {
                    s.push_str(spacing_text);
                }
            }
        }
        s
    }

    /// Validates a header with the provided reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - The pak file format reader object.
    /// * `raw` - Raw header data to validate.
    /// * `encryption_key` - AES key for decryption.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the header is valid.
    fn validate_header_with_reader(
        reader: &AAPakFileFormatReader,  // Placeholder for the pak file format reader type
        raw: &[u8],
        encryption_key: &[u8],
    ) -> bool {
        let data = match Self::decrypt_aes(raw, encryption_key, false) {
            Some(d) => d,
            None => return false,
        };

        let mut cursor = 0;
        let mut f_count = 0u32;
        let mut e_count = 0u32;

        // Helper functions to read data from the byte array
        fn read_byte(data: &[u8], cursor: &mut usize) -> u8 {
            let res = data[*cursor];
            *cursor += 1;
            res
        }

        fn read_u32(data: &[u8], cursor: &mut usize) -> u32 {
            let res = u32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap());
            *cursor += 4;
            res
        }

        for &element in &reader.read_order {
            // Check if still inside valid range
            if cursor >= data.len() {
                return false;
            }

            match element {
                AAPakFileHeaderElement::AnyByte => {
                    let _ = read_byte(&data, &mut cursor);
                }
                AAPakFileHeaderElement::NullByte => {
                    let zero = read_byte(&data, &mut cursor);
                    if zero != 0 {
                        return false; // Expected value is not 0x00
                    }
                }
                AAPakFileHeaderElement::Header => {
                    for i in 0..reader.header_bytes.len() {
                        let b = read_byte(&data, &mut cursor);
                        if b != reader.header_bytes[i] {
                            return false; // Invalid header string data
                        }
                    }
                }
                AAPakFileHeaderElement::FilesCount => {
                    f_count = read_u32(&data, &mut cursor);
                }
                AAPakFileHeaderElement::ExtraFilesCount => {
                    e_count = read_u32(&data, &mut cursor);
                }
            }
        }

        // Assuming these are fields of some structure
        OWNER.pak_type = PakFileType::Reader;
        KEY = reader.header_encryption_key.clone();
        FILE_COUNT = f_count;
        EXTRA_FILE_COUNT = e_count;

        true
    }

    // Placeholder for AES decryption function
    fn decrypt_aes(raw: &[u8], key: &[u8], do_encryption: bool) -> Option<Vec<u8>> {
        // Implement AES decryption logic here
        // For example, using the `aes` crate for Rust:
        use aes::Aes128;
        use aes::cipher::{NewBlockCipher, BlockDecrypt, BlockEncrypt, BlockCipher};
        use block_modes::BlockMode;
        use block_modes::cbc::Cbc;
        use block_padding::Padding;

        const IV: &[u8; 16] = &[0; 16]; // Example IV, replace as needed

        let cipher = if do_encryption {
            Cbc::<Aes128, Padding::No>::new_from_slices(key, IV).unwrap()
        } else {
            Cbc::<Aes128, Padding::No>::new_from_slices(key, IV).unwrap()
        };

        let mut buffer = raw.to_vec();
        let result = if do_encryption {
            cipher.encrypt(&mut buffer, raw.len()).ok()
        } else {
            cipher.decrypt(&mut buffer).ok()
        };

        result.map(|_| buffer)
    }

    /// Decrypts the current header data to get the file counts depending on header type.
    pub(crate) fn decrypt_header_data(&self,
                                      // raw_data: &[u8],  // Raw header data
                                      // key: &mut [u8],   // AES key
                                      // owner: &mut Owner, // Owner with reader and other properties
                                      // reader_pool: &[AAPakFileFormatReader], // Reader pool to guess the reader
    ) {
        // Save key that has been set by the program (if any), and try options using this one first
        let pre_defined_key = self.key.to_vec();

        // If assigned a reader manually, check that one first
        if let Some(reader) = &self.owner.reader {
            if pre_defined_key.len() == 16 {
                if validate_header_with_reader(reader, self.raw_data, &pre_defined_key) {
                    owner.is_valid = true;
                    return;
                }
            }

            if validate_header_with_reader(reader, raw_data, &reader.header_encryption_key) {
                *key = reader.header_encryption_key.clone(); // override loaded key
                owner.is_valid = true;
                return;
            }
        }

        // Try guessing what reader to use from the ReaderPool
        for reader_check in reader_pool {
            if pre_defined_key.len() == 16 {
                if validate_header_with_reader(reader_check, raw_data, &pre_defined_key) {
                    owner.reader = Some(reader_check.clone());
                    owner.is_valid = true;
                    return;
                }
            }

            if validate_header_with_reader(reader_check, raw_data, &reader_check.header_encryption_key) {
                owner.reader = Some(reader_check.clone());
                *key = reader_check.header_encryption_key.clone(); // override loaded key
                owner.is_valid = true;
                return;
            }
        }

        // Fallback to default to verify
        let data = decrypt_aes(raw_data, key, false);

        if let Some(data) = data {
            // A valid header/footer check by its identifier
            if data.starts_with(&['W' as u8, 'I' as u8, 'B' as u8, 'O' as u8]) {
                // W I B O = 0x57 0x49 0x42 0x4F
                owner.pak_type = PakFileType::Classic;
                owner.file_count = u32::from_le_bytes(data[8..12].try_into().unwrap());
                owner.extra_file_count = u32::from_le_bytes(data[12..16].try_into().unwrap());
                owner.is_valid = true;
            } else {
                // Doesn't look like this is a pak file, the file is corrupted, or is in an unknown layout/format
                owner.file_count = 0;
                owner.extra_file_count = 0;
                owner.is_valid = false;

                if owner.debug_mode {
                    let hex = byte_array_to_hex_string(key, "", "");
                    std::fs::write(format!("game_pak_failed_header_{}.key", hex), &data).unwrap();
                }
            }
        } else {
            owner.file_count = 0;
            owner.extra_file_count = 0;
            owner.is_valid = false;
        }
    }

    /// Encrypts the current header data.
    fn encrypt_header_data(
        data: &mut Vec<u8>,  // Data buffer to write header data
        file_count: u32,    // Number of files
        extra_file_count: u32, // Number of extra files
        header_size: usize, // Size of the header
        owner: &Owner,      // Owner with pak type and reader
        key: &[u8],         // AES key
    ) {
        let mut ms = Cursor::new(vec![0u8; header_size]);
        let mut writer = Cursor::new(vec![0u8; header_size]);

        // Write initial data to the memory stream
        writer.write_all(&data[..header_size]).expect("Write to memory stream failed");

        match owner.pak_type {
            PakFileType::Classic => {
                writer.set_position(0);
                writer.write_all(&[b'W', b'I', b'B', b'O']).expect("Write to memory stream failed");
                writer.set_position(8);
                writer.write_u32_le(file_count).expect("Write to memory stream failed");
                writer.set_position(12);
                writer.write_u32_le(extra_file_count).expect("Write to memory stream failed");
            }
            PakFileType::Reader => {
                if let Some(reader) = &owner.reader {
                    writer.set_position(0);
                    for read_order in &reader.read_order {
                        match read_order {
                            AAPakFileHeaderElement::AnyByte | AAPakFileHeaderElement::NullByte => {
                                writer.write_all(&[0]).expect("Write to memory stream failed");
                            }
                            AAPakFileHeaderElement::Header => {
                                writer.write_all(&reader.header_bytes).expect("Write to memory stream failed");
                            }
                            AAPakFileHeaderElement::FilesCount => {
                                writer.write_u32_le(file_count).expect("Write to memory stream failed");
                            }
                            AAPakFileHeaderElement::ExtraFilesCount => {
                                writer.write_u32_le(extra_file_count).expect("Write to memory stream failed");
                            }
                            _ => panic!("Unexpected header element"),
                        }
                    }
                } else {
                    panic!("Owner reader is not set");
                }
            }
            _ => panic!("Unsupported PakFileType"),
        }

        // Copy the modified data back
        ms.set_position(0);
        let len = ms.read(&mut data[..header_size]).expect("Read from memory stream failed");

        // Encrypt our stored data into raw_data
        let encrypted_data = Self::encrypt_aes(&data[..len], key, true).expect("AES encryption failed");
        *data = encrypted_data;
    }
}


fn main() {
    let owner = AAPak {
        gp_file_stream: File::create_new("test.pak").unwrap(),
        file_count: 0,
        extra_file_count: 0,
        fat: vec![],
        first_file_info_offset: 0,
    };
    let mut header = AAPakFileHeader::new(owner);

    // Example usage of `set_custom_key`
    header.set_custom_key([
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
    ]);

    println!("Header Size: {}", header.size);
}

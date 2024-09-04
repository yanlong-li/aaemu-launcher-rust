use crate::pak::aa_pak_file_format_reader::AAPakFileFormatReader;
use crate::pak::aa_pak_file_format_reader::AAPakFileInfoElement::Md5;
use crate::pak::aa_pak_file_header::AAPakFileHeader;
use crate::pak::aa_pak_file_info::AAPakFileInfo;
use crate::pak::aa_pak_loading_progress_type::{AAPakLoadingProgressType, AAPakNotify};
use crate::pak::io::SeekFrom;
use crate::pak::packer_sub_stream::PackerSubStream;
use crate::pak::pak_file_type::PakFileType;
use chrono::{TimeZone, Utc};
use hex::FromHex;
use std::cmp::PartialEq;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AAPak {
    // Current Reader that is handling headers
    pub(crate) reader: Option<AAPakFileFormatReader>,

    // List of available readers that will be used to automatically detect the format
    pub reader_pool: Vec<AAPakFileFormatReader>,

    // Points to this pakFile's header
    pub header: AAPakFileHeader,

    // When enabled, failing to open a pakFile will dump the currently used key as a file
    pub debug_mode: bool,

    // List of all unused files
    pub(crate) extra_files: Vec<AAPakFileInfo>,

    // List of all used files
    pub(crate) files: Vec<AAPakFileInfo>,

    // Virtual list of all folder names
    pub folders: Vec<String>,

    // Set to true if there have been changes made that require a rewrite of the FAT and/or header
    pub is_dirty: bool,

    // Checks if current pakFile information is loaded into memory
    pub is_open: bool,

    // Set to true if this is not a pak file, but rather information loaded from somewhere else
    pub is_virtual: bool,

    // Returns the newest file time of the files inside the pakFile
    pub newest_file_date: SystemTime,

    // Virtual data to return as a null value for file details
    pub null_aapak_file_info: AAPakFileInfo,

    // If set to true, adds the freed space from a delete to the previous file's padding
    pub padding_delete_mode: bool,

    // Which pak style is being used
    pub(crate) pak_type: PakFileType,

    // Flag to enable automatic MD5 recalculations when adding or replacing a file
    pub auto_update_md5_when_adding: bool,

    pub on_progress: dyn AAPakNotify,

    // Defines how many files reading from FAT are skipped between OnProgress events
    pub on_progress_fat_file_interval: usize,

    // Internally used PakFile filename
    pub gp_file_path: Option<String>,

    // The Internally used FileStream when a pakFile is open
    pub gp_file_stream: Option<File>,

    // Show if this pakFile is opened in read-only mode
    pub read_only: bool,

    // Returns the text message of the last internally caught exception
    pub last_error: String,
}

impl PartialEq for &AAPakFileInfo {
    fn eq(&self, other: &Self) -> bool {
        self.md5 == other.md5
    }
}

impl AAPak {
    /// Creates a game_pak file handler
    pub fn new() -> Self {
        AAPak {
            reader: Some(AAPakFileFormatReader::new(true)),
            reader_pool: vec![AAPakFileFormatReader::new(true)],
            header: AAPakFileHeader::new(None),
            debug_mode: false,
            extra_files: Vec::new(),
            files: Vec::new(),
            folders: Vec::new(),
            is_dirty: false,
            is_open: false,
            is_virtual: false,
            newest_file_date: SystemTime::UNIX_EPOCH,
            null_aapak_file_info: AAPakFileInfo::new(
                0,
                0,
                0,
                0,
                0,
                [0; 16],
                0,
                "".to_string(),
                0,
                0,
                0,
                0,
            ),
            padding_delete_mode: false,
            pak_type: PakFileType::Classic,
            auto_update_md5_when_adding: true,
            on_progress: (),
            on_progress_fat_file_interval: 10_000,
            gp_file_path: None,
            gp_file_stream: None,
            read_only: false,
            last_error: String::new(),
        }
    }


    /// Opens a pak file, can only be used if no other file is currently loaded
    pub fn open_pak(&mut self, file_path: &str, open_as_read_only: bool) -> bool {
        self.trigger_progress(AAPakLoadingProgressType::OpeningFile, 0, 100);

        // Fail if already open
        if self.is_open {
            return false;
        }

        // Check if file exists
        if !Path::new(file_path).exists() {
            return false;
        }

        self.is_virtual = false;

        self.trigger_progress(AAPakLoadingProgressType::OpeningFile, 25, 100);
        let ext = Path::new(file_path).extension().and_then(std::ffi::OsStr::to_str).unwrap_or("").to_lowercase();
        let res;

        if ext == "csv" {
            self.read_only = true;
            // Open file as CSV data
            res = self.open_virtual_csv_pak(file_path);
            self.trigger_progress(AAPakLoadingProgressType::OpeningFile, 100, 100);
            return res;
        }

        match OpenOptions::new()
            .read(true)
            .write(!open_as_read_only)
            .create(false)
            .open(file_path) {
            Ok(file_stream) => {
                self.gp_file_path = Some(file_path.to_string());
                self.gp_file_stream = Some(file_stream);
                self.is_dirty = false;
                self.is_open = true;
                self.read_only = open_as_read_only;
                res = self.read_header();
            }
            Err(e) => {
                self.gp_file_path = None;
                self.gp_file_stream = None;
                self.is_open = false;
                self.read_only = true;
                self.last_error = e.to_string();
                res = false;
            }
        }
        self.trigger_progress(AAPakLoadingProgressType::OpeningFile, 100, 100);
        res
    }

    /// Creates a new pak file with the given filename, overwriting if it exists
    pub fn new_pak(&mut self, file_path: &str) -> bool {
        self.trigger_progress(AAPakLoadingProgressType::OpeningFile, 0, 100);

        // Fail if already open
        if self.is_open {
            return false;
        }

        self.is_virtual = false;

        let res;
        match File::create(file_path) {
            Ok(file_stream) => {
                self.gp_file_path = Some(file_path.to_string());
                self.gp_file_stream = Some(file_stream);
                self.read_only = false;
                self.is_open = true;
                self.is_dirty = true;
                self.save_header(); // Save blank data
                res = self.read_header(); // read blank data to confirm
            }
            Err(e) => {
                self.gp_file_path = None;
                self.gp_file_stream = None;
                self.is_open = false;
                self.read_only = true;
                self.last_error = e.to_string();
                res = false;
            }
        }
        self.trigger_progress(AAPakLoadingProgressType::OpeningFile, 100, 100);
        res
    }

    /// Opens a CSV file as a virtual pak file
    pub fn open_virtual_csv_pak(&mut self, csv_file_path: &str) -> bool {
        // Fail if already open
        if self.is_open {
            return false;
        }

        // Check if file exists
        if !Path::new(csv_file_path).exists() {
            return false;
        }

        self.is_virtual = true;
        self.gp_file_stream = None; // Not used on virtual pakFiles
        self.gp_file_path = Some(csv_file_path.to_string());
        self.is_dirty = false;
        self.is_open = true;
        self.read_only = true;
        self.pak_type = PakFileType::Csv;

        match self.read_csv_data() {
            Ok(_) => {
                true
            }
            Err(e) => {
                self.is_open = false;
                self.read_only = true;
                self.last_error = e.to_string();
                false
            }
        }
    }

    /// Closes the currently opened pakFile (if open)
    pub fn close_pak(&mut self) {
        self.trigger_progress(AAPakLoadingProgressType::ClosingFile, 0, 100);

        if !self.is_open {
            return;
        }

        if self.is_dirty && !self.read_only {
            self.save_header();
        }

        if let Some(stream) = self.gp_file_stream.take() {
            if let Err(e) = stream.sync_all() {
                // Handle synchronization error if needed
                eprintln!("Error synchronizing file before closing: {}", e);
            }
        }

        self.gp_file_path = None;
        self.is_open = false;
        self.pak_type = PakFileType::Classic;
        self.reader = None;
        self.header.set_default_key();
        self.last_error.clear();

        self.trigger_progress(AAPakLoadingProgressType::ClosingFile, 100, 100);
    }

    /// Encrypts and saves the Header and File Information Table back to the pak.
    /// This is also automatically called on close_pak() if changes were made.
    /// Warning: Failing to save will corrupt your pak if files were added or deleted!
    pub fn save_header(&mut self) -> bool {
        self.trigger_progress(AAPakLoadingProgressType::WritingHeader, 0, 100);

        let result = std::panic::catch_unwind(|| {
            if let Err(e) = self.header.write_to_fat() {
                self.last_error = e.to_string();
                return false;
            }

            self.trigger_progress(AAPakLoadingProgressType::WritingHeader, 50, 100);

            if let Some(stream) = self.gp_file_stream.as_mut() {
                if let Err(e) = stream.seek(SeekFrom::Start(self.header.first_file_info_offset as u64)) {
                    self.last_error = e.to_string();
                    return false;
                }

                if let Err(e) = self.header.fat.seek(SeekFrom::Start(0)) {
                    self.last_error = e.to_string();
                    return false;
                }

                if let Err(e) = std::io::copy(&mut self.header.fat, stream) {
                    self.last_error = e.to_string();
                    return false;
                }

                if let Err(e) = stream.set_len(stream.stream_position().unwrap()) {
                    self.last_error = e.to_string();
                    return false;
                }
            } else {
                self.last_error = "File stream is not open".to_string();
                return false;
            }

            self.is_dirty = false;

            self.trigger_progress(AAPakLoadingProgressType::WritingHeader, 100, 100);
            true
        });

        match result {
            Ok(success) => success,
            Err(_) => {
                self.last_error = "Panic occurred while saving header".to_string();
                false
            }
        }
    }

    /// Reads the Pak Header and FAT
    ///
    /// # Returns
    /// Returns true if the read information makes a valid pakFile
    pub fn read_header(&mut self) -> bool {
        self.trigger_progress(AAPakLoadingProgressType::ReadingHeader, 0, 100);

        // Reset internal state
        self.newest_file_date = SystemTime::UNIX_EPOCH;
        self.files.clear();
        self.extra_files.clear();
        self.folders.clear();

        // Seek to the end of the file and read the last 512 bytes
        if let Some(stream) = self.gp_file_stream.as_mut() {
            if let Err(e) = stream.seek(SeekFrom::End(-(AAPakFileHeader::SIZE as i64))) {
                self.last_error = e.to_string();
                return false;
            }

            self.trigger_progress(AAPakLoadingProgressType::ReadingHeader, 10, 100);

            let mut raw_data = [0u8; AAPakFileHeader::SIZE];
            let amount_read = match stream.read(&mut raw_data) {
                Ok(n) => n,
                Err(e) => {
                    self.last_error = e.to_string();
                    return false;
                }
            };

            if amount_read < 32 {
                return false;
            }

            self.trigger_progress(AAPakLoadingProgressType::ReadingHeader, 25, 100);

            // Decrypt the header data
            self.header.decrypt_header_data();

            self.trigger_progress(AAPakLoadingProgressType::ReadingHeader, 50, 100);

            if self.header.is_valid {
                // Only allow editing for Classic
                // if self.pak_type != PakFileType::PakTypeA {
                //     self.read_only = true;
                // }

                self.header.load_raw_fat().unwrap();
                self.header.read_file_table();
            } else {
                self.header.fat.set_position(0);
                self.header.fat.get_mut().clear();
            }

            self.trigger_progress(AAPakLoadingProgressType::ReadingHeader, 100, 100);

            self.header.is_valid
        } else {
            self.last_error = "File stream is not open".to_string();
            false
        }
    }

    /// Converts a hex string to a byte array
    ///
    /// # Arguments
    ///
    /// * `hex` - A string slice that holds the hex string. Must be a multiple of 2 in length.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the byte array.
    ///
    /// # Panics
    ///
    /// This function will panic if the hex string is not a valid hex string or is not a multiple of 2 in length.
    pub fn string_to_byte_array(hex: &str) -> Vec<u8> {
        Vec::from_hex(hex).expect("Invalid hex string or length not a multiple of 2")
    }

    /// Converts a specialized string to a file time as UTC
    ///
    /// # Arguments
    ///
    /// * `encoded_string` - A string that contains the date and time in the format "yyyyMMddHHmmss".
    ///
    /// # Returns
    ///
    /// An `i64` representing the file time in UTC, or 0 if parsing fails.
    pub fn date_time_str_to_file_time(encoded_string: &str) -> i64 {
        if encoded_string.len() < 18 {
            return 0; // Invalid format length
        }

        let year = match encoded_string.get(0..4).and_then(|s| s.parse::<i32>().ok()) {
            Some(y) => y,
            None => return 0,
        };

        let month = match encoded_string.get(5..7).and_then(|s| s.parse::<u32>().ok()) {
            Some(m) => m,
            None => return 0,
        };

        let day = match encoded_string.get(8..10).and_then(|s| s.parse::<u32>().ok()) {
            Some(d) => d,
            None => return 0,
        };

        let hour = match encoded_string.get(11..13).and_then(|s| s.parse::<u32>().ok()) {
            Some(h) => h,
            None => return 0,
        };

        let minute = match encoded_string.get(14..16).and_then(|s| s.parse::<u32>().ok()) {
            Some(m) => m,
            None => return 0,
        };

        let second = match encoded_string.get(17..19).and_then(|s| s.parse::<u32>().ok()) {
            Some(s) => s,
            None => return 0,
        };

        let dt = match Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
            .single()
        {
            Some(d) => d,
            None => return 0,
        };

        dt.timestamp()
    }

    // fn from_csv_record(record: &csv::StringRecord) -> Result<Self, Box<dyn Error>> {
    //     if record.len() != 10 {
    //         return Err("Invalid number of fields".into());
    //     }
    //
    //     let size = record[1].parse()?;
    //     let offset = record[2].parse()?;
    //     let md5 = hex::decode(&record[3])?;
    //     let create_time = Self::date_time_str_to_file_time(&record[4]);
    //     let modify_time = Self::date_time_str_to_file_time(&record[5]);
    //     let size_duplicate = record[6].parse()?;
    //     let padding_size = record[7].parse()?;
    //     let dummy1 = record[8].parse()?;
    //     let dummy2 = record[9].parse()?;
    //
    //     Ok(AAPakFileInfo {
    //         name: record[0].to_string(),
    //         size,
    //         offset,
    //         md5,
    //         create_time,
    //         modify_time,
    //         size_duplicate,
    //         padding_size,
    //         dummy1,
    //         dummy2,
    //         deleted_index_number: 0,
    //         entry_index_number: 0,
    //     })
    // }

    fn read_csv_data(&mut self) -> bool {
        self.trigger_progress(AAPakLoadingProgressType::ReadingFAT, 0, 100);
        self.files.clear();
        self.extra_files.clear();
        self.folders.clear();


        let file_path = self.gp_file_path.clone().unwrap();

        let lines = match std::fs::read_to_string(file_path) {
            Ok(content) => content.lines().collect::<Vec<_>>(),
            Err(_) => return false,
        };

        self.trigger_progress(AAPakLoadingProgressType::ReadingFAT, 0, lines.len() as i32);

        if !lines.is_empty() {
            let csv_head = "Name;Size;Offset;Md5;CreateTime;ModifyTime;SizeDuplicate;PaddingSize;Dummy1;Dummy2";

            self.header.is_valid = lines[0].to_lowercase() != csv_head;
        } else {
            self.header.is_valid = false;
        }

        self.trigger_progress(AAPakLoadingProgressType::ReadingFAT, 1, lines.len() as i32);

        if !self.header.is_valid {
            return self.header.is_valid;
        }

        for (i, line) in lines.iter().skip(1).enumerate() {
            let fields: Vec<&str> = line.split(';').collect();
            if fields.len() != 10 {
                continue;
            }
            let fni = match (
                fields[0].to_string(),
                fields[1].parse(),
                fields[2].parse(),
                <[u8; 16]>::from(Self::string_to_byte_array(fields[3])),
                Self::date_time_str_to_file_time(fields[4]),
                Self::date_time_str_to_file_time(fields[5]),
                fields[6].parse(),
                fields[7].parse(),
                fields[8].parse(),
                fields[9].parse()
            ) {
                (name, Ok(size), Ok(offset), md5, create_time, modify_time, Ok(size_duplicate), Ok(padding_size), Ok(dummy1), Ok(dummy2)) => AAPakFileInfo {
                    name,
                    size,
                    offset,
                    md5,
                    create_time,
                    modify_time,
                    size_duplicate,
                    padding_size,
                    dummy1,
                    dummy2,
                    deleted_index_number: 0,
                    entry_index_number: 0,
                },
                _ => {
                    self.header.is_valid = false;
                    self.last_error = "Failed to parse file info".to_string();
                    return false;
                }
            };

            self.files.push(fni);

            if i % self.on_progress_fat_file_interval == 0 {
                self.trigger_progress(AAPakLoadingProgressType::ReadingFAT, i as i32, lines.len() as i32);
            }
        }

        self.trigger_progress(AAPakLoadingProgressType::ReadingFAT, lines.len() as i32, lines.len() as i32);

        self.header.is_valid
    }


    /// Find a file information inside the pak by its filename
    ///
    /// # Arguments
    /// * `filename` - Filename inside the pak of the requested file
    /// * `file_info` - Returns the AAPakFileInfo of the requested file or NullAAPakFileInfo if it does not exist
    ///
    /// # Returns
    /// Returns true if the file was found
    pub fn get_file_by_name(&self, filename: &str) -> (bool, AAPakFileInfo) {
        let file_info = self.files.iter()
            .find(|pfi| pfi.name == filename)
            .unwrap_or(&self.null_aapak_file_info)
            .clone();  // Assuming AAPakFileInfo implements Clone

        (file_info != None, file_info)
    }

    /// <summary>
    /// Exports a given file as a Stream
    /// </summary>
    /// <param name="file">AAPakFileInfo of the file to be exported</param>
    /// <returns>Returns a PackerSubStream of file within the pak</returns>
    pub fn export_file_as_stream(&mut self, file: &AAPakFileInfo) -> PackerSubStream<File> {
        let pos = file.stream_position().unwrap();
        let size = file.metadata().unwrap().len();
        PackerSubStream::new(self.gp_file_stream.take(), pos, size).unwrap()
    }

    /// <summary>
    /// Calculates and set the MD5 Hash of a given file
    /// </summary>
    /// <param name="file">AAPakFileInfo of the file to be updated</param>
    /// <returns>Returns the new hash as a hex string (with removed dashes)</returns>
    pub fn update_md5(&mut self, file: &mut AAPakFileInfo) -> String {
        let mut hasher = Md5::new();
        let mut stream = self.export_file_as_stream(file);
        let mut buffer = [0u8; 1024];
        while let Ok(n) = stream.read(&mut buffer) {
            if n == 0 { break; }
            hasher.update(&buffer[..n]);
        }

        let new_hash = hasher.finalize();
        let new_hash_bytes = new_hash.as_slice();

        if file.md5 != new_hash_bytes {
            // Only update if different
            file.md5.copy_from_slice(new_hash_bytes);
            self.is_dirty = true;
        }

        hex::encode(new_hash_bytes)
    }

    /// <summary>
    /// Try to find a file inside the pak file based on an offset position inside the pak file.
    /// Note: this only checks inside the used files and does not account for "deleted" files
    /// </summary>
    /// <param name="offset">Offset to check against</param>
    /// <param name="file_info">Returns the found file's info, or NullAAPakFileInfo if nothing was found</param>
    /// <returns>Returns true if the location was found to be inside a valid file</returns>
    pub fn find_file_by_offset(&self, offset: i64) -> Option<&AAPakFileInfo> {
        for pfi in &self.files {
            if offset >= pfi.offset && offset <= pfi.offset + pfi.size + pfi.padding_size {
                return Some(pfi);
            }
        }
        None
    }

    /// <summary>
    /// Replaces a file's data with new data from a stream, can only be used if the current file location has enough space
    /// to hold the new data
    /// </summary>
    /// <param name="pfi">FileInfo of the file to replace</param>
    /// <param name="source_stream">Stream to replace the data with</param>
    /// <param name="modify_time">Time to be used as a modified time stamp</param>
    /// <returns>Returns true on success</returns>
    pub fn replace_file(
        &mut self,
        pfi: &mut AAPakFileInfo,
        mut source_stream: impl Read,
        modify_time: SystemTime,
    ) -> bool {
        // Overwrite an existing file in the pak

        if self.read_only {
            return false;
        }

        // Fail if the new file is too big
        let source_stream_len = match source_stream.by_ref().bytes().count() {
            Ok(len) => len as i64,
            Err(_) => return false,
        };

        if source_stream_len > pfi.size + pfi.padding_size {
            return false;
        }

        // Save end_pos for easy calculation later
        let end_position = pfi.offset + pfi.size + pfi.padding_size;

        // Try to copy new data over the old data
        let mut gp_file_stream = &self.gp_file_stream;
        gp_file_stream.seek(SeekFrom::Start(pfi.offset as u64)).ok()?;
        std::io::copy(&mut source_stream, &mut gp_file_stream).ok()?;

        // Update File Size in File Table
        pfi.size = source_stream_len;
        pfi.size_duplicate = pfi.size;
        // Calculate new Padding Size
        pfi.padding_size = (end_position - pfi.size - pfi.offset) as i32;

        // Recalculate the MD5 hash
        if self.auto_update_md5_when_adding {
            self.update_md5(pfi);
        }

        pfi.modify_time = modify_time.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;

        pfi.dummy1 = self.reader.as_ref().map_or(0, |r| r.default_dummy1);
        pfi.dummy2 = self.reader.as_ref().map_or(0, |r| r.default_dummy2);

        // Mark File Table as dirty
        self.is_dirty = true;

        true
    }


    /// <summary>
    /// Delete a file from pak. Behaves differently depending on the paddingDeleteMode setting
    /// </summary>
    /// <param name="pfi">AAPakFileInfo of the file that is to be deleted</param>
    /// <returns>Returns true on success</returns>
    pub fn delete_file(&mut self, pfi: &AAPakFileInfo) -> bool {
        // When we delete a file from the pak, we remove the entry from the FileTable and expand the previous file's padding to take up the space
        if self.read_only {
            return false;
        }

        if self.padding_delete_mode {
            if let Some(prev_pfi) = self.find_file_by_offset(pfi.offset - 1) {
                let mut prev_pfi = prev_pfi.to_owned();
                // If we have a previous file, expand its padding area with the free space from this file
                prev_pfi.padding_size = prev_pfi.padding_size + pfi.size + pfi.padding_size;
            }
            self.files.retain(|file| file != pfi);
        } else {
            // "move" Offset and Size data to extraFiles
            let e_file = AAPakFileInfo {
                create_time: 0,
                name: "__unused__".to_string(),
                offset: pfi.offset,
                size: pfi.size + pfi.padding_size,
                size_duplicate: pfi.size + pfi.padding_size,
                padding_size: 0,
                md5: [0; 16],
                dummy1: self.reader.as_ref().map_or(0, |r| r.default_dummy1),
                dummy2: self.reader.as_ref().map_or(0, |r| r.default_dummy2 as u64),
                deleted_index_number: 0,
                entry_index_number: 0,
                modify_time: 0,
            };

            self.extra_files.push(e_file);

            self.files.retain(|file| file != pfi);
        }

        self.is_dirty = true;
        true
    }

    /// <summary>
    /// Adds a new file into the pak
    /// </summary>
    /// <param name="filename">Filename of the file inside the pakFile</param>
    /// <param name="sourceStream">Source Stream containing the file data</param>
    /// <param name="createTime">Time to use as initial file creation timestamp</param>
    /// <param name="modifyTime">Time to use as last modified timestamp</param>
    /// <param name="auto_spare_space">When set, tries to pre-allocate extra free space at the end of the file, this will be 25%
    /// of the fileSize if used. If a "deleted file" is used, this parameter is ignored
    /// </param>
    /// <param name="pfi">Returns the fileInfo of the newly created file</param>
    /// <returns>Returns true on success</returns>
    pub fn add_as_new_file(
        &mut self,
        filename: &str,
        source_stream: &mut dyn Read,
        create_time: SystemTime,
        modify_time: SystemTime,
        auto_spare_space: bool,
    ) -> Result<AAPakFileInfo, ()> {
        // When we have a new file, or previous space wasn't enough, we will add it where the file table starts, and move the file table
        if self.read_only {
            return Err(());
        }

        let mut added_at_the_end = true;

        let mut new_file = AAPakFileInfo {
            name: filename.to_string(),
            offset: self.header.first_file_info_offset,
            size: source_stream.seek(SeekFrom::End(0)).unwrap() as i64,
            size_duplicate: 0,
            create_time: create_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            modify_time: modify_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            padding_size: 0,
            md5: [0; 16],
            dummy1: self.reader.as_ref().map_or(0, |r| r.default_dummy1),
            dummy2: self.reader.as_ref().map_or(0, |r| r.default_dummy2 as u64),
            deleted_index_number: 0,
            entry_index_number: 0,
        };

        // check if we have "unused" space in extra_files that we can use
        for i in 0..self.extra_files.len() {
            if new_file.size <= self.extra_files[i].size {
                // Copy the spare file's properties and remove it from extra_files
                new_file.offset = self.extra_files[i].offset;
                new_file.padding_size = self.extra_files[i].size - new_file.size; // This should already be aligned
                added_at_the_end = false;
                self.extra_files.remove(i);
                break;
            }
        }

        if added_at_the_end {
            // Only need to calculate padding if we are adding at the end
            let dif = new_file.size % 0x200;
            if dif > 0 {
                new_file.padding_size = 0x200 - dif;
            }
            if auto_spare_space {
                // If auto_spare_space is used to add files, we will reserve some extra space as padding
                // Add 25% by default
                let spare_space = new_file.size / 4;
                let aligned_spare_space = spare_space - spare_space % 0x200;
                new_file.padding_size += aligned_spare_space;
            }
        }

        // Add to files list
        self.files.push(new_file.clone());

        self.is_dirty = true;


        let mut gp_file_stream = self.gp_file_stream.take().unwrap();

        // Add File Data
        gp_file_stream.seek(SeekFrom::Start(new_file.offset as u64)).unwrap();
        source_stream.seek(SeekFrom::Start(0)).unwrap();
        io::copy(source_stream, &mut gp_file_stream).unwrap();

        if added_at_the_end {
            self.header.first_file_info_offset = new_file.offset + new_file.size + new_file.padding_size;
        }

        // TODO: optimize this to calculate WHILE we are copying the stream
        if self.auto_update_md5_when_adding {
            self.update_md5(&mut new_file);
        }

        // Set output
        Ok(new_file)
    }

    fn add_file_from_file(
        &mut self,
        source_file_name: &str,
        as_file_name: &str,
        auto_spare_space: bool,
    ) -> bool {
        if !Path::new(source_file_name).exists() {
            return false;
        }

        match File::open(source_file_name) {
            Ok(file) => {
                let create_time = file.metadata().unwrap().created().unwrap();
                let mod_time = file.metadata().unwrap().modified().unwrap();
                self.add_file_from_stream(as_file_name, &file, create_time, mod_time, auto_spare_space).expect("TODO: panic message");


                true
            }
            Err(_) => false,
        }
    }

    /// <summary>
    /// Adds or replaces a given file with Name filename with data from sourceStream
    /// </summary>
    /// <param name="filename">The filename used inside the pak</param>
    /// <param name="sourceStream">Source Stream of file to be added</param>
    /// <param name="createTime">Time to use as original file creation time</param>
    /// <param name="modifyTime">Time to use as last modified time</param>
    /// <param name="autoSpareSpace">Enable adding 25% of the sourceStream Size as padding when not replacing a file</param>
    /// <param name="pfi">AAPakFileInfo of the newly added or modified file</param>
    /// <returns>Returns true on success</returns>
    pub fn add_file_from_stream(
        &mut self,
        filename: &str,
        source_stream: &mut dyn Read,
        create_time: SystemTime,
        modify_time: SystemTime,
        auto_spare_space: bool,
    ) -> Result<AAPakFileInfo, ()> {
        if self.read_only {
            return Err(());
        }
        let mut pfi = AAPakFileInfo::default();

        let mut add_as_new = true;
        // Try to find the existing file
        if self.get_file_by_name(filename) {
            let reserved_size_max = pfi.size + pfi.padding_size;
            add_as_new = source_stream.seek(SeekFrom::End(0)).unwrap() > reserved_size_max as u64;
            // Bug-fix: If we have insufficient space, make sure to delete the old file first as well
            if add_as_new {
                self.delete_file(&pfi);
            }
        }

        if add_as_new {
            self.add_as_new_file(filename, source_stream, create_time, modify_time, auto_spare_space)?;
        } else {
            self.replace_file(&mut pfi, source_stream, modify_time);
        }

        Ok(pfi)
    }


    /// <summary>
    /// Helper function to report progress
    /// </summary>
    /// <param name="progress_type">Type of progress being reported</param>
    /// <param name="step">Current step in the progress</param>
    /// <param name="maximum">Maximum steps for this progress</param>
    pub(crate) fn trigger_progress(
        &self,
        progress_type: AAPakLoadingProgressType,
        step: i32,
        maximum: i32,
    ) {
        if let Some(on_progress) = &self.on_progress {
            on_progress(self, progress_type, step, maximum);
        }
    }
}

impl Drop for AAPak {
    fn drop(&mut self) {
        self.close_pak();
    }
}

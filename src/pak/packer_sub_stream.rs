use std::io::{self, Read, Seek, SeekFrom};

pub struct PackerSubStream<R: Read + Seek> {
    base_stream: R,
    base_offset: u64,
    length: u64,
    position: u64,
}

impl<R: Read + Seek> PackerSubStream<R> {
    pub fn new(mut base_stream: R, offset: u64, length: u64) -> io::Result<Self> {
        if offset < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "offset must be non-negative"));
        }

        let mut buffer = [0; 512];
        if base_stream.seek(SeekFrom::Start(offset))? != offset {
            let mut remaining = offset;
            while remaining > 0 {
                let read = base_stream.read(&mut buffer[..remaining as usize])?;
                remaining -= read as u64;
            }
        }

        Ok(PackerSubStream {
            base_stream,
            base_offset: offset,
            length,
            position: 0,
        })
    }
}

impl<R: Read + Seek> Read for PackerSubStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = self.length - self.position;
        if remaining == 0 {
            return Ok(0);
        }

        let count = buf.len() as u64;
        let count = if count > remaining {
            remaining as usize
        } else {
            count as usize
        };

        let bytes_read = self.base_stream.read(&mut buf[..count])?;
        self.position += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl<R: Read + Seek> Seek for PackerSubStream<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Seek not supported"))
    }
}

impl<R: Read + Seek> io::Write for PackerSubStream<R> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Write not supported"))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.base_stream.flush()
    }
}

impl<R: Read + Seek> PackerSubStream<R> {
    pub fn length(&self) -> u64 {
        self.length
    }

    pub fn position(&self) -> u64 {
        self.position
    }

    pub fn set_position(&mut self, position: u64) {
        self.position = if position > self.length {
            self.length
        } else {
            position
        };
        let _ = self.base_stream.seek(SeekFrom::Start(self.base_offset + self.position));
    }
}

// fn main() -> io::Result<()> {
//     let data = b"Hello, world! This is a test.";
//     let cursor = io::Cursor::new(data);
//
//     let mut stream = PackerSubStream::new(cursor, 7, 5)?;
//     let mut buf = [0; 5];
//     stream.read(&mut buf)?;
//     println!("Read bytes: {:?}", &buf);
//
//     Ok(())
// }

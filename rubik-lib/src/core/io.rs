use std::{
    fs::File,
    io::{BufReader, BufWriter, ErrorKind::InvalidData, Read, Write},
    path::Path,
};

pub trait BinarySerde: Sized {
    fn to_binary(&self) -> Vec<u8>;
    fn from_binary(buffer: &[u8]) -> Option<Self>;

    fn to_file(&self, file: impl AsRef<Path>) -> std::io::Result<()> {
        let mut writer = BufWriter::new(File::create(file)?);
        writer.write_all(&self.to_binary())
    }

    fn from_file(file: impl AsRef<Path>) -> std::io::Result<Self> {
        let mut reader = BufReader::new(File::open(file)?);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Self::from_binary(&buffer).ok_or(InvalidData.into())
    }
}

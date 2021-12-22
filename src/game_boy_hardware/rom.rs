use std::{fs::File, io::Read};

pub struct Rom
{
    pub file: File
}

impl Rom
{
    pub fn new(f: File) -> Rom
    {
        Rom
        {
            file: f
        }
    }

    pub fn get_file_bytes(&mut self) -> Vec<u8>
    {
        let mut result:Vec<u8> = Vec::new();
        let mut buf = Vec::<u8>::new();
        self.file.read_to_end(&mut buf).unwrap();
        for byte in buf
        {
            result.push(byte);
        }
        result
    }
}
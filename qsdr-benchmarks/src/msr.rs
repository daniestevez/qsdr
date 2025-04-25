use anyhow::Result;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

pub struct Msr {
    file: File,
    cpu_number: usize,
}

const APERF_ADDR: u64 = 0xe8;

impl Msr {
    pub fn new(cpu_number: usize) -> Result<Msr> {
        Ok(Msr {
            file: File::open(format!("/dev/cpu/{cpu_number}/msr"))?,
            cpu_number,
        })
    }

    fn read_register(&mut self, address: u64) -> Result<u64> {
        let mut buf = [0u8; std::mem::size_of::<u64>()];
        self.file.seek(SeekFrom::Start(address))?;
        self.file.read_exact(&mut buf)?;
        let value = u64::from_ne_bytes(buf);
        Ok(value)
    }

    pub fn read_aperf(&mut self) -> Result<u64> {
        self.read_register(APERF_ADDR)
    }

    pub fn cpu_number(&self) -> usize {
        self.cpu_number
    }
}

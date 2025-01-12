use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc::{self, CRC_32_ISO_HDLC};
use serde_derive::{Deserialize, Serialize};

type ByteString = Vec<u8>;
type ByteStr = [u8];

const CRC32: crc::Crc<u32> = crc::Crc::<u32>::new(&CRC_32_ISO_HDLC);

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyValuePair {
    pub key: ByteString,
    pub value: ByteString,
}

#[derive(Debug)]
pub struct ActionKV {
    f: File,
    pub index: HashMap<ByteString, u64>,
}

impl ActionKV {
    pub fn open(path: &Path) -> io::Result<Self> {
        println!("Openning the file");
        let f = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;
        println!("Creating the index");
        let index = HashMap::new();
        Ok(ActionKV { f, index })
    }

    pub fn seek_to_end(&mut self) -> io::Result<u64> {
        self.f.seek(io::SeekFrom::End(0))
    }

    pub fn load(&mut self) -> io::Result<()> {
        print!("Loading data");
        let mut f = BufReader::new(&mut self.f);

        loop {
            let current_position = f.seek(io::SeekFrom::Current(0))?;
            println!("Processing record at {:?}", current_position);
            let maybe_kv = ActionKV::process_record(&mut f);

            let kv = match maybe_kv {
                Ok(kv) => kv,
                Err(err) => match err.kind() {
                    io::ErrorKind::UnexpectedEof => {
                        break;
                    }
                    _ => return Err(err),
                },
            };
            self.index.insert(kv.key, current_position);
        }

        Ok(())
    }

    pub fn get(&mut self, key: &ByteStr) -> io::Result<Option<ByteString>> {
        println!("Looking up the key");
        let position = match self.index.get(key) {
            None => {
                println!("Key not found");
                return Ok(None);
            }
            Some(pos) => {
                println!("Key found, value is at {:?}", pos);
                *pos
            }
        };

        let kv = self.get_at(position)?;
        println!("kv fetched");
        Ok(Some(kv.value))
    }

    pub fn get_at(&mut self, position: u64) -> io::Result<KeyValuePair> {
        let mut f = BufReader::new(&mut self.f);
        f.seek(io::SeekFrom::Start(position))?;

        let kv = ActionKV::process_record(&mut f)?;
        Ok(kv)
    }

    pub fn find(&mut self, target: &ByteStr) -> io::Result<Option<(u64, ByteString)>> {
        let mut f = BufReader::new(&mut self.f);
        let mut found: Option<(u64, ByteString)> = None;

        loop {
            let position = f.seek(io::SeekFrom::Current(0))?;
            let maybe_kv = ActionKV::process_record(&mut f);
            let kv = match maybe_kv {
                Ok(kv) => kv,
                Err(err) => match err.kind() {
                    io::ErrorKind::UnexpectedEof => {
                        break;
                    }
                    _ => return Err(err),
                },
            };

            if kv.key == target {
                found = Some((position, kv.key));
            }
        }

        Ok(found)
    }

    pub fn insert(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<u64> {
        let position = 0;
        let mut f = BufWriter::new(&mut self.f);
        let mut new_entry_layout = ByteString::with_capacity(key.len() + value.len());

        for byte in key {
            new_entry_layout.push(*byte);
        }
        for byte in value {
            new_entry_layout.push(*byte);
        }

        let checksum = CRC32.checksum(&new_entry_layout);
        let current_position = f.seek(SeekFrom::End(0))?;
        f.write_u32::<LittleEndian>(checksum)?;
        f.write_u32::<LittleEndian>(key.len() as u32)?;
        f.write_u32::<LittleEndian>(value.len() as u32)?;
        f.write_all(&new_entry_layout)?;

        self.index.insert(key.to_vec(), position);

        Ok(current_position)
    }

    #[inline]
    pub fn update(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<()> {
        match self.insert(key, value) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    #[inline]
    pub fn delete(&mut self, key: &ByteStr) -> io::Result<()> {
        match self.insert(key, b"") {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn process_record<R: Read>(f: &mut R) -> io::Result<KeyValuePair> {
        let checksum32 = f.read_u32::<LittleEndian>()?;
        let key_len = f.read_u32::<LittleEndian>()?;
        let value_len = f.read_u32::<LittleEndian>()?;
        let data_len = key_len + value_len;
        let mut data = ByteString::with_capacity(data_len as usize);

        {
            f.by_ref().take(data_len as u64).read_to_end(&mut data)?;
        }
        debug_assert_eq!(data_len, data.len() as u32);

        let calculated_checksum = CRC32.checksum(&data);

        if checksum32 != calculated_checksum {
            panic!(
                "Checksum mismatch ({:08x} and {:08x})",
                checksum32, calculated_checksum
            );
        }

        let value = data.split_off(key_len as usize);
        let key = data;

        Ok(KeyValuePair { key, value })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn read_test() {
//         let akv = ActionKV {
//
//         }
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }

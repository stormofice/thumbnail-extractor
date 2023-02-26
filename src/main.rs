mod file_identification;

use std::fmt::{Display, Formatter};
use std::fs;
use std::mem::transmute;
use std::ops::Deref;
use crate::file_identification::{determine_file_type, FileIdentification};

const MAGIC: [u8; 4] = [0x43, 0x4D, 0x4D, 0x4D];

struct DatabaseHeader {
    magic: [u8; 4],
    version: u32,
    cache_type: u32,
    unknown: u32,
    first_entry_offset: u32,
    available_entry_offset: u32,
}

const DATABASE_HEADER_SIZE: usize = std::mem::size_of::<DatabaseHeader>();

impl Display for DatabaseHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DatabaseHeader {{ magic: {:?}, version: {}, cache_type: {}, unknown: {}, first_entry_offset: {}, available_entry_offset: {} }}",
               { self.magic }, { self.version }, { self.cache_type }, { self.unknown }, { self.first_entry_offset }, { self.available_entry_offset })
    }
}

#[repr(C, packed)]
struct CacheEntry {
    magic: [u8; 4],
    entry_size: u32,
    hash: u64,
    filename_length: u32,
    padding_size: u32,
    data_size: u32,
    width: u32,
    height: u32,
    unknown: u32,
    data_checksum: u64,
    header_checksum: u64,
}

const CACHE_ENTRY_SIZE: usize = std::mem::size_of::<CacheEntry>();

impl Display for CacheEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CacheEntry {{ magic: {:?}, entry_size: {}, hash: {:x}, filename_length: {}, padding_size: {}, data_size: {}, width: {}, height: {}, unknown: {}, data_checksum: {:x}, header_checksum: {:x} }}",
               { self.magic }, { self.entry_size }, { self.hash }, { self.filename_length }, { self.padding_size }, { self.data_size }, { self.width }, { self.height }, { self.unknown }, { self.data_checksum }, { self.header_checksum })
    }
}

#[derive(Debug)]
enum CacheType {
    Size16x16 = 0x0,
    Size32x32 = 0x1,
    Size48x48 = 0x2,
    Size96x96 = 0x3,
    Size256x256 = 0x4,
    Size768x768 = 0x5,
    Size1280x1280 = 0x6,
    Size1920x1920 = 0x7,
    Size2560x2560 = 0x8,
    Sr = 0x9,
    Wide = 0xA,
    Exif = 0xB,
    WideAlternate = 0xC,
    CustomStream = 0xD,
}

impl TryFrom<u32> for CacheType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(CacheType::Size16x16),
            0x1 => Ok(CacheType::Size32x32),
            0x2 => Ok(CacheType::Size48x48),
            0x3 => Ok(CacheType::Size96x96),
            0x4 => Ok(CacheType::Size256x256),
            0x5 => Ok(CacheType::Size768x768),
            0x6 => Ok(CacheType::Size1280x1280),
            0x7 => Ok(CacheType::Size1920x1920),
            0x8 => Ok(CacheType::Size2560x2560),
            0x9 => Ok(CacheType::Sr),
            0xA => Ok(CacheType::Wide),
            0xB => Ok(CacheType::Exif),
            0xC => Ok(CacheType::WideAlternate),
            0xD => Ok(CacheType::CustomStream),
            _ => Err(()),
        }
    }
}

fn main() -> std::io::Result<()> {
    let contents = fs::read("./thumbcache_1280.db")?;

    println!("File size: {}", contents.len());
    println!("Database header size: {}", DATABASE_HEADER_SIZE);
    println!("Cache entry size: {}", CACHE_ENTRY_SIZE);

    let header = unsafe {
        transmute::<[u8; DATABASE_HEADER_SIZE],
            DatabaseHeader>(contents[0..DATABASE_HEADER_SIZE]
            .try_into().expect("Invalid cache entry"))
    };

    println!("{}", header);
    assert_eq!(header.magic, MAGIC);

    fs::create_dir_all("output")?;

    let mut offset = header.first_entry_offset as usize;
    while offset < contents.len() {
        println!("Entry at offset {}", offset);

        let entry = unsafe {
            transmute::<[u8; CACHE_ENTRY_SIZE],
                CacheEntry>(contents[offset..offset + CACHE_ENTRY_SIZE]
                .try_into().expect("Invalid cache entry"))
        };


        // println!("{}", entry);
        assert_eq!(entry.magic, MAGIC);

        let (_, filename, _) = unsafe {
            contents[(offset + CACHE_ENTRY_SIZE)..
                (offset + CACHE_ENTRY_SIZE + entry.filename_length as usize)]
                .align_to::<u16>()
        };

        // println!("Filename: {:?}", String::from_utf16_lossy(filename));

        let data_start = offset + CACHE_ENTRY_SIZE + entry.filename_length as usize + entry.padding_size as usize;
        let data_end = data_start + entry.data_size as usize;
        assert!(data_end <= contents.len());
        let data = &contents[data_start..data_end];

        if data.len() == 0 {
            println!("Finished?");
            break;
        }

        let ident = determine_file_type(data);
        match ident {
            None => eprintln!("Could not determine file type {:?}", &data[0..16]),
            Some(file_identification) => println!("File type is {:?}", file_identification.file_type),
        }

        // let path = format!("output/{}.jpg", { entry.hash });
        // fs::write(path, data).expect("Unable to write file");

        offset += entry.entry_size as usize;
    }

    Ok(())
}

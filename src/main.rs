use std::fs;
use std::mem::transmute;

struct DatabaseHeader {
    magic: [u8; 4],
    version: u32,
    cache_type: u32,
    unknown: u32,
    first_entry_offset: u32,
    available_entry_offset: u32,
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
    println!("Database header size: {}", std::mem::size_of::<DatabaseHeader>());
    println!("Cache entry size: {}", CACHE_ENTRY_SIZE);

    let (prefix, header, suffix) = unsafe {
        contents.align_to::<DatabaseHeader>()
    };

    println!("Prefix length: {:?}", { prefix.len() });
    println!("Suffix length: {:?}", { suffix.len() });

    let header = &header[0];

    println!("Magic: {:?}", header.magic);
    println!("Version: {}", header.version);
    println!("Cache type: {:?}", CacheType::try_from(header.cache_type).unwrap());

    println!("Unknown: {}", header.unknown);
    println!("First entry offset: {}", header.first_entry_offset);
    println!("Available entry offset: {}", header.available_entry_offset);

    assert_eq!(header.magic, [0x43, 0x4D, 0x4D, 0x4D]);

    // Offset until now: 24 bytes
    let mut offset = header.first_entry_offset as usize;

    while offset < contents.len()  {
        println!("Entry at offset {}",  offset);

        let entry = unsafe {
            transmute::<[u8; CACHE_ENTRY_SIZE],
                CacheEntry>(contents[offset..offset + CACHE_ENTRY_SIZE]
                .try_into().expect("Invalid cache entry"))
        };

        println!("Magic: {:?}", { entry.magic });
        println!("Entry size: {}", { entry.entry_size });
        println!("Hash: {:x}", { entry.hash });
        println!("Filename length: {}", { entry.filename_length });
        println!("Padding size: {}", { entry.padding_size });
        println!("Data size: {}", { entry.data_size });
        println!("Width: {}", { entry.width });
        println!("Height: {}", { entry.height });
        println!("Unknown: {}", { entry.unknown });
        println!("Data checksum: {:x}", { entry.data_checksum });
        println!("Header checksum: {:x}", { entry.header_checksum });

        assert_eq!(entry.magic, [0x43, 0x4D, 0x4D, 0x4D]);

        let (_, filename, _) = unsafe {
            contents[(offset + CACHE_ENTRY_SIZE)..
                (offset + CACHE_ENTRY_SIZE + entry.filename_length as usize)]
                .align_to::<u16>()
        };
        println!("Filename: {:?}", String::from_utf16_lossy(filename));

        let data_start = offset + CACHE_ENTRY_SIZE + entry.filename_length as usize + entry.padding_size as usize;
        let data_end = data_start + entry.data_size as usize;
        let data = &contents[data_start..data_end];
        //println!("Data: {:x?}", data);

        //fs::write("sample.jpg", data).expect("Unable to write file");

        offset += entry.entry_size as usize;
    }

    Ok(())
}

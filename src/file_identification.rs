#[derive(Debug, PartialEq)]
pub enum FileType {
    JPEG,
    PNG,
    BMP,
    GIF,
}

pub struct FileIdentification {
    pub(crate) file_type: FileType,
    pub(crate) file_extension: &'static str,
    file_header: &'static [u8],
}

const FILE_MAPPINGS: [FileIdentification; 7] = [
    FileIdentification {
        file_type: FileType::JPEG,
        file_extension: "jpg",
        file_header: &[0xFF, 0xD8, 0xFF, 0xD8],
    },
    FileIdentification {
        file_type: FileType::JPEG,
        file_extension: "jpg",
        file_header: &[0xFF, 0xD8, 0xFF, 0xEE],
    },
    FileIdentification {
        file_type: FileType::JPEG,
        file_extension: "jpg",
        file_header: &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01],
    },
    FileIdentification {
        file_type: FileType::BMP,
        file_extension: "bmp",
        file_header: &[0x42, 0x4D],
    },
    FileIdentification {
        file_type: FileType::PNG,
        file_extension: "png",
        file_header: &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
    },
    FileIdentification {
        file_type: FileType::GIF,
        file_extension: "gif",
        file_header: &[0x47, 0x49, 0x46, 0x38, 0x37, 0x61],
    },
    FileIdentification {
        file_type: FileType::GIF,
        file_extension: "gif",
        file_header: &[0x47, 0x49, 0x46, 0x38, 0x39, 0x61],
    },
];

const MAX_HEADER_LENGTH: usize = 128;

pub fn determine_file_type(mut header: &[u8]) -> Option<&FileIdentification> {
    if header.len() > MAX_HEADER_LENGTH {
        header = &header[0..MAX_HEADER_LENGTH];
    }
    FILE_MAPPINGS.iter().find(|&mapping| header.starts_with(mapping.file_header))
}

#[cfg(test)]
mod file_type_tests {
    use crate::file_identification::{determine_file_type};
    use crate::file_identification::FileType::JPEG;

    #[test]
    fn test_determine_file_type_jpeg() {
        let test_data: &[u8] = &[0xFF, 0xD8, 0xFF, 0xD8];

        let file_identification = determine_file_type(test_data);
        assert!(file_identification.is_some());
        assert_eq!(file_identification.unwrap().file_type, JPEG);
    }
}
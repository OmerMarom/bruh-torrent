use bencode;

struct File {
    length: u64,
    path: String,
}

enum FilesData {
    SingleFile {
        length: u64
    },
    MultipleFiles {
        files: Vec<File>
    }
}

struct Info {
    name: Option<String>,
    piece_length: u32,
    pieces: String,
    file_data: FilesData
}

struct MetaInfo {
    announce: String,
    info: Info
}

pub fn parse(content: &[u8]) {
    // TODO Understand why this operation can fail.
    let bencode = bencode::from_buffer(content).unwrap();

    let mut decoder = bencode::Decoder::new(&bencode);

    let result = 
}


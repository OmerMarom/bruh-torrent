use std::{fs, io};
use thiserror::Error;
use sha1::{Sha1, Digest};
use hex;

use crate::bencode;

pub struct File {
    pub path: Vec<String>,
    pub length: usize,
}

pub struct Info {
    pub hash: String,
    pub name: Option<String>,
    pub piece_length: usize,
    pub pieces: Vec<Vec<u8>>,
    pub files: Vec<File>,
}

pub struct TorrentInfo {
    pub announce: String,
    pub info: Info,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    FileError(#[from] io::Error),
    #[error("Invalid bencode")]
    InvalidBencode,
    #[error("Missing field {0}")]
    MissingField(&'static str),
    #[error("Invalid pieces field")]
    InvalidPieces,
}

pub fn parse(filepath: &str) -> Result<TorrentInfo, ParseError> {
    let bencode_content = fs::read(filepath)?;
    
    let root = bencode::parse(&bencode_content)
        .ok_or(ParseError::InvalidBencode)?;

    let root_dict = root.as_dictionary()
        .ok_or(ParseError::MissingField("root"))?;

    let announce = root_dict.get("announce")
        .and_then(|announce| announce.as_str())
        .ok_or(ParseError::MissingField("announce"))?
        .to_string();

    let info_value = root_dict.get("info")
        .ok_or(ParseError::MissingField("info"))?;

    // TODO Is recreating the hasher for each announce ok?
    let mut hasher = Sha1::new();
    hasher.update(info_value.unparsed);
    let hash = hex::encode(hasher.finalize().as_slice());

    let info_dict = info_value.as_dictionary()
        .ok_or(ParseError::MissingField("info"))?;

    let name = info_dict.get("name")
        .and_then(|name| name.as_str())
        .map(|name| name.to_string());

    let piece_length = info_dict.get("piece length")
        .and_then(|piece_length| piece_length.as_integer())
        .ok_or(ParseError::MissingField("piece length"))?
        .clone() as usize;

    let pieces_byte_str = info_dict.get("pieces")
        .and_then(|pieces| pieces.as_byte_string())
        .ok_or(ParseError::MissingField("pieces"))?;

    const PIECE_SIZE: usize = 20;

    if pieces_byte_str.len() % PIECE_SIZE != 0 {
        return Err(ParseError::InvalidPieces);
    }

    let mut pieces = Vec::new();
    let mut pieces_left = &pieces_byte_str[..];
    while !pieces_left.is_empty() {
        pieces.push(pieces_left[..PIECE_SIZE].to_vec());
        pieces_left = &pieces_left[PIECE_SIZE..];
    }

    let files = 
        if let Some(length) =
            info_dict.get("length")
                .and_then(|length| length.as_integer()) {
            
            vec![File {
                path: vec![
                    name.as_ref()
                        .map_or(String::from("Default name"), |name| name.clone())
                ],
                length: length.clone() as usize
            }]
        } else {
            info_dict.get("files")
                .and_then(|files| files.as_list())
                .ok_or(ParseError::MissingField("length/files"))?
                .iter().map(|file_value| {
                    let file_dict = file_value.as_dictionary()
                        .ok_or(ParseError::MissingField("file"))?;

                    let path = file_dict.get("path")
                        .and_then(|path| path.as_list())
                        .ok_or(ParseError::MissingField("path"))?
                        .iter().map(|path_item| Some(path_item.as_str()?.to_string()))
                        .collect::<Option<Vec<String>>>()
                        .ok_or(ParseError::MissingField("path item"))?;

                    let length = file_dict.get("length")
                        .and_then(|length| length.as_integer())
                        .ok_or(ParseError::MissingField("length"))? as usize;

                    Ok(File { path, length })
                })
                .collect::<Result<Vec<File>, ParseError>>()?
        };

    Ok(TorrentInfo {
        announce,
        info: Info {
            hash, 
            name,
            piece_length,
            pieces,
            files,
        },
    })
}


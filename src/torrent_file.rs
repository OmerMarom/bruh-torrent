use std::{fs, io};
use thiserror::Error;

use crate::bencode;

pub struct File {
    pub path: String,
    pub length: usize,
}

pub struct Info {
    pub name: Option<String>,
    pub piece_length: usize,
    pub pieces: Vec<String>,
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
    let bencode_content = fs::read_to_string(filepath)?;
    
    let content_value = bencode::parse(&bencode_content)
        .ok_or(ParseError::InvalidBencode)?;
    let content_dict = content_value.as_dictionary()
        .ok_or(ParseError::MissingField("root"))?;

    let announce = content_dict.get("announce")
        .and_then(|announce| announce.as_string())
        .ok_or(ParseError::MissingField("announce"))?
        .clone();

    let info_dict = content_dict.get("info")
        .and_then(|info| info.as_dictionary())
        .ok_or(ParseError::MissingField("root"))?;

    let name = info_dict.get("name")
        .and_then(|name| name.as_string())
        .map(|name| name.clone());

    let piece_length = info_dict.get("piece length")
        .and_then(|piece_length| piece_length.as_integer())
        .ok_or(ParseError::MissingField("piece length"))?
        .clone() as usize;

    let pieces_str = info_dict.get("pieces")
        .and_then(|pieces| pieces.as_string())
        .ok_or(ParseError::MissingField("pieces"))?;

    const PIECE_SIZE: usize = 20;

    if pieces_str.len() % PIECE_SIZE != 0 {
        return Err(ParseError::InvalidPieces);
    }

    let mut pieces = Vec::new();
    let mut pieces_str_left = &pieces_str[..];
    while !pieces_str_left.is_empty() {
        pieces.push(pieces_str_left[..PIECE_SIZE].to_owned());
        pieces_str_left = &pieces_str_left[..PIECE_SIZE];
    }

    let files = 
        if let Some(length) =
            info_dict.get("length")
                .and_then(|length| length.as_integer()) {
            
            vec![File {
                path: name.as_ref().map_or(String::from("Default name"), |name| name.clone()),
                length: length.clone() as usize
            }]
        } else {
            let files = info_dict.get("files")
                .and_then(|files| files.as_list())
                .ok_or(ParseError::MissingField("length/files"))?
                .iter()
                .map(|file_value| {
                    let file_dict = file_value.as_dictionary()
                        .ok_or(ParseError::MissingField("file"))?;

                    let path = file_dict.get("path")
                        .and_then(|path| path.as_string())
                        .ok_or(ParseError::MissingField("path"))?;

                    let length = file_dict.get("length")
                        .and_then(|length| length.as_integer())
                        .ok_or(ParseError::MissingField("length"))?;

                    Ok(File { 
                        path: path.clone(),
                        length: length.clone() as usize
                    })
                })
                .collect::<Result<Vec<File>, ParseError>>()?;
            
            files 
        };

    Ok(TorrentInfo {
        announce,
        info: Info {
            name,
            piece_length,
            pieces,
            files,
        },
    })
}


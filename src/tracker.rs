use std::{fmt, str, time::Duration};
use reqwest;
use thiserror::Error;
use urlencoding;
use url::form_urlencoded;

use crate::bencode;

pub enum AnnounceEvent {
    Started,
    Completed,
    Stopped,
}

impl fmt::Display for AnnounceEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnnounceEvent::Started => write!(f, "{}", "started"),
            AnnounceEvent::Completed => write!(f, "{}", "completed"),
            AnnounceEvent::Stopped => write!(f, "{}", "stopped")
        }
    }
}

pub struct AnnounceParams {
    pub info_hash: [u8; 20],
    pub peer_id: String,
    pub port: u16,
    pub uploaded: usize,
    pub downloaded: usize,
    pub left: usize,
    pub event: AnnounceEvent,
}

pub struct Peer {
    id: String,
    ip: String,
    port: u16,
}

pub struct AnnounceResponse {
    interval: Duration,
    peers: Vec<Peer>
}

#[derive(Error, Debug)]
pub enum AnnounceError {
    #[error("{0}")]
    Http(#[from] reqwest::Error),
    #[error("Response contains invalid bencode: {0}")]
    InvalidBencode(#[from] bencode::ParseError),
    #[error("Response missing field {0}")]
    MissingField(&'static str),
    #[error("Response contains negative interval")]
    NegativeInterval,
    #[error("Tracker responded with error: {0}")]
    ErrorResponse(String),
}

pub async fn announce(announce: &str, params: &AnnounceParams) -> Result<AnnounceResponse, AnnounceError> {
    // Reqwest does not support encoding the info hash as bytes so we encode it manually.

    let params_without_info_hash = [
        ("peer_id", params.peer_id.clone()),
        ("port", params.port.to_string()),
        ("uploaded", params.uploaded.to_string()),
        ("downloaded", params.downloaded.to_string()),
        ("left", params.left.to_string()),
        ("event", params.event.to_string()),
    ];
    let encoded_params_without_info_hash = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(params_without_info_hash.iter())
        .finish();
    let encoded_info_hash = urlencoding::encode_binary(&params.info_hash).into_owned();
    let url = format!("{}?{}&info_hash={}", announce, encoded_params_without_info_hash, encoded_info_hash);
 
    let request = reqwest::Client::new().get(url);
    let bencode_response = request.send().await?.bytes().await?;

    println!("Tracker response: {}", str::from_utf8(&bencode_response).unwrap());

    let response_value = bencode::parse(&bencode_response)?;
 
    println!("Tracker parsed response: {}", response_value);

    let response_dict = response_value
        .as_dictionary()
        .ok_or(AnnounceError::MissingField("root"))?;

    if let Some(failure_reason) =
        response_dict.get("failure reason")
        .and_then(|failure_reason| failure_reason.as_str()) {

        return Err(AnnounceError::ErrorResponse(failure_reason.to_string()))
    }

    let interval = Duration::from_secs(
        response_dict.get("interval")
            .and_then(|interval| interval.as_integer())
            .ok_or(AnnounceError::MissingField("interval"))
            .and_then(|interval| {
                if interval < 0 {
                    Err(AnnounceError::NegativeInterval)
                } else {
                    Ok(interval as u64)
                }
            })?
        );

    let peers = response_dict.get("peers")
        .and_then(|peer_values| peer_values.as_list())
        .ok_or(AnnounceError::MissingField("peers"))?
        .iter()
        .map(|peer_value| {
            let peer_dict = peer_value.as_dictionary()
                .ok_or(AnnounceError::MissingField("peer"))?;

            let id = peer_dict.get("peer id")
                .and_then(|id| id.as_str())
                .ok_or(AnnounceError::MissingField("peer id"))?
                .to_string();

            let ip = peer_dict.get("ip")
                .and_then(|ip| ip.as_str())
                .ok_or(AnnounceError::MissingField("ip"))?
                .to_string();

            let port = peer_dict.get("port")
                .and_then(|port| port.as_integer())
                .ok_or(AnnounceError::MissingField("port"))?
                .clone() as u16;

            Ok(Peer { id, ip, port })
        })
    .collect::<Result<Vec<Peer>, AnnounceError>>()?;

    Ok(AnnounceResponse { interval, peers })
}


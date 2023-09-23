mod torrent_file;
mod tracker;
mod bencode;

#[tokio::main]
async fn main() {
    const TORRENT_FILE: &str = "/home/omer/Downloads/adventuresofsher00doylrich_archive.torrent";
    const PEER_ID: &str = "OMERMAROM69420694206";
    const PORT: u16 = 10_000;

    let torrent_info = torrent_file::parse(TORRENT_FILE).unwrap();

    println!("Parsed torrent file.");

    let torrent_length = torrent_info.info.files
        .iter().fold(0, |length, file| file.length + length);

    let request = tracker::AnnounceParams {
        info_hash: String::from("123"),
        peer_id: String::from(PEER_ID),
        port: PORT,
        uploaded: 0,
        downloaded: 0,
        left: torrent_length,
        event: tracker::AnnounceEvent::Started
    };
    tracker::announce(&torrent_info.announce, &request).await.unwrap();
}


mod torrent_file;
mod tracker;
mod bencode;

#[tokio::main]
async fn main() {
    const TORRENT_FILE: &str = "/home/omer/Downloads/ubuntu-23.10-beta-live-server-amd64.iso.torrent";
    const PEER_ID: &str = "ABCDEFGHIJKLMNOPQRST";
    const PORT: u16 = 6881;

    let torrent_info = torrent_file::parse(TORRENT_FILE).unwrap();

    println!("Parsed torrent file.");

    let torrent_length = torrent_info.info.files
        .iter().fold(0, |length, file| file.length + length);

    let request = tracker::AnnounceParams {
        info_hash: torrent_info.info.hash,
        peer_id: String::from(PEER_ID),
        port: PORT,
        uploaded: 0,
        downloaded: 0,
        left: torrent_length,
        event: tracker::AnnounceEvent::Started
    };
    tracker::announce(&torrent_info.announce, &request).await.unwrap();
}


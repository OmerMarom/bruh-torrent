mod torrent_file;
mod tracker;

#[tokio::main]
async fn main() {
    const TORRENT_FILE: &str = "~/omer/dev/bruhtorrent/some_torrent.torrent";
    const PEER_ID: &str = "1234567890";
    const PORT: u16 = 10_000;

    let torrent_info = torrent_file::parse(TORRENT_FILE).unwrap();

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


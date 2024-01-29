use listen1::provider::kugou::Kugou;
use listen1::provider::kugou::KuGouPlaylistParams;
use listen1::provider::netease::{Netease, NeteasePlaylistsParams};


#[tokio::main]
async fn main() {
    env_logger::init();
    // let kugou = Kugou::new();
    //
    // let netease = Netease::default();
    // let album_id = 1552283;
    // let resp = netease.get_album(album_id).await;

    let netease = Netease::default();
    let params = NeteasePlaylistsParams {
        category_id: "".to_string(),
        order: "hot".to_string(),
        limit: 10,
        offset: Some(0),
    };
    let playlists = netease.get_playlists(params).await;
}

use serde::Serialize;

#[derive(Serialize)]
enum PlaylistOrder {
    Hot,
    New,
}

#[derive(Serialize)]
struct KwPlaylistParams {
    offset: u32,
    rn: u8,
    order: PlaylistOrder,
    #[serde(rename(serialize = "camelCase"))]
    https_status: u8,
    pn: u32,
}

#[cfg(test)]
mod tests {
    #[test]
    fn x() {

    }
}
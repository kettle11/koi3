#[cfg(not(target_arch = "wasm32"))]
pub async fn fetch_bytes(path: &str) -> Result<Vec<u8>, ()> {
    #[cfg(feature = "network_requests")]
    let contents = if path.starts_with("http") {
        use std::io::Read;

        let response = ureq::get(path).call().map_err(|_| ())?;

        let mut bytes: Vec<u8> = Vec::new();
        response
            .into_reader()
            .take(10_000_000)
            .read_to_end(&mut bytes)
            .map_err(|_| ())?;
        bytes
    } else {
        std::fs::read(path).map_err(|_| ())?;
    };
    #[cfg(not(feature = "network_requests"))]
    let contents = std::fs::read(path).map_err(|_| ())?;
    Ok(contents)
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_bytes(path: &str) -> Result<Vec<u8>, ()> {
    kwasm::libraries::fetch(path).await
}

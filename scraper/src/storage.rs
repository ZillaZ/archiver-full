pub async fn download_thumbnail(url: &str, path: &str) -> Result<(), String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("failed to download thumbnail: {e}"))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("failed to read thumbnail bytes: {e}"))?;

    tokio::fs::write(path, &bytes)
        .await
        .map_err(|e| format!("failed to write thumbnail: {e}"))?;

    Ok(())
}
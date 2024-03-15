use base64::Engine;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Base64Image {
    pub mime_type: String,
    pub data: String,
}

impl Base64Image {
    pub fn to_data_uri(&self) -> String {
        format!("data:{};base64,{}", self.mime_type, self.data)
    }
}

pub async fn load_image_from_url(url: &str) -> anyhow::Result<Base64Image> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    let mime_type = response
        .headers()
        .get("Content-Type")
        .and_then(|ct| ct.to_str().ok())
        .map(|ct| ct.to_lowercase())
        .unwrap_or_else(|| "image/jpeg".to_owned());

    const SUPPORTED_MIME_TYPES: &[&str] = &["image/png", "image/jpeg", "image/jpg"];
    if !SUPPORTED_MIME_TYPES.contains(&mime_type.as_str()) {
        return Err(anyhow::anyhow!("Unsupported image format: {}", mime_type));
    }

    let response_bytes = response.bytes().await?;

    let engine = base64::engine::general_purpose::STANDARD;
    let base64_encoded = engine.encode(&response_bytes);

    Ok(Base64Image {
        mime_type,
        data: base64_encoded,
    })
}

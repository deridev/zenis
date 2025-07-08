use base64::Engine;
use mime::Mime;

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

    let content_type = response
        .headers()
        .get("Content-Type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("application/octet-stream");

    let mime: Mime = content_type.parse()?;

    let mime_type = if mime.type_() == mime::IMAGE {
        match mime.subtype().as_str() {
            "png" => "image/png",
            "jpeg" | "jpg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg+xml" => "image/svg+xml",
            "tiff" => "image/tiff",
            "bmp" => "image/bmp",
            "x-icon" => "image/x-icon",
            other => other, // Allow other image subtypes
        }
    } else {
        return Err(anyhow::anyhow!(
            "Unsupported content type: {}",
            content_type
        ));
    };

    let response_bytes = response.bytes().await?;

    let engine = base64::engine::general_purpose::STANDARD;
    let base64_encoded = engine.encode(&response_bytes);

    Ok(Base64Image {
        mime_type: mime_type.to_string(),
        data: base64_encoded,
    })
}

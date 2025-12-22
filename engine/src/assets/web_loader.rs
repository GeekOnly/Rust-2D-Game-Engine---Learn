use engine_core::assets::AssetLoader;
use anyhow::{Result, Context, anyhow};
use async_trait::async_trait;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

pub struct WebAssetLoader {
    base_url: String,
}

impl WebAssetLoader {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    async fn fetch(&self, path: &str) -> Result<Response> {
        let window = web_sys::window().ok_or_else(|| anyhow!("No window found"))?;
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| anyhow!("Failed to create request: {:?}", e))?;

        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await
            .map_err(|e| anyhow!("Fetch failed: {:?}", e))?;

        let resp: Response = resp_value.dyn_into()
            .map_err(|_| anyhow!("Response is not a Response object"))?;

        if !resp.ok() {
            return Err(anyhow!("Fetch error: {} {}", resp.status(), resp.status_text()));
        }

        Ok(resp)
    }
}

#[async_trait]
impl AssetLoader for WebAssetLoader {
    async fn load_text(&self, path: &str) -> Result<String> {
        let resp = self.fetch(path).await?;
        let text = JsFuture::from(resp.text().map_err(|e| anyhow!("Failed to get text: {:?}", e))?).await
            .map_err(|e| anyhow!("Failed to resolve text promise: {:?}", e))?;
        
        text.as_string().ok_or_else(|| anyhow!("Response text was not a string"))
    }

    async fn load_binary(&self, path: &str) -> Result<Vec<u8>> {
        let resp = self.fetch(path).await?;
        let array_buffer = JsFuture::from(resp.array_buffer().map_err(|e| anyhow!("Failed to get array buffer: {:?}", e))?).await
            .map_err(|e| anyhow!("Failed to resolve array buffer promise: {:?}", e))?;
        
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        Ok(uint8_array.to_vec())
    }

    fn get_base_path(&self) -> String {
        self.base_url.clone()
    }
}

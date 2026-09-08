#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
pub mod web {
    use wasm_bindgen::JsCast;

    /// Descarrega directament un fitxer al navegador del client utilitzant Blob i <a download>
    pub fn download_file(file_name: &str, mime_type: &str, content: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
        let window = web_sys::window().ok_or_else(|| wasm_bindgen::JsValue::from_str("No window available"))?;
        let document = window.document().ok_or_else(|| wasm_bindgen::JsValue::from_str("No document available"))?;

        let uint8_array = js_sys::Uint8Array::new_with_length(content.len() as u32);
        uint8_array.copy_from(content);
        let array = js_sys::Array::new();
        array.push(&uint8_array);

        let options = web_sys::BlobPropertyBag::new();
        options.set_type(mime_type);
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&array, &options)?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)?;

        let anchor = document
            .create_element("a")?
            .dyn_into::<web_sys::HtmlAnchorElement>()?;
        anchor.set_href(&url);
        anchor.set_download(file_name);

        let body = document.body().ok_or_else(|| wasm_bindgen::JsValue::from_str("No body element"))?;
        body.append_child(&anchor)?;
        anchor.click();
        anchor.remove();
        let _ = web_sys::Url::revoke_object_url(&url);

        Ok(())
    }

    /// Obre un document HTML en una pestanya nova per visualitzar o imprimir
    pub fn open_html_in_new_tab(html_content: &str) -> Result<(), wasm_bindgen::JsValue> {
        let window = web_sys::window().ok_or_else(|| wasm_bindgen::JsValue::from_str("No window available"))?;
        let array = js_sys::Array::new();
        array.push(&wasm_bindgen::JsValue::from_str(html_content));

        let options = web_sys::BlobPropertyBag::new();
        options.set_type("text/html;charset=utf-8");
        let blob = web_sys::Blob::new_with_str_sequence_and_options(&array, &options)?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)?;

        let _ = window.open_with_url_and_target(&url, "_blank")?;
        Ok(())
    }

    const STORAGE_KEY: &str = "nutricio_app_state";

    /// Guarda l'estat en format JSON al localStorage del navegador
    pub fn save_to_local_storage(json_str: &str) -> Result<(), wasm_bindgen::JsValue> {
        let window = web_sys::window().ok_or_else(|| wasm_bindgen::JsValue::from_str("No window available"))?;
        if let Ok(Some(storage)) = window.local_storage() {
            storage.set_item(STORAGE_KEY, json_str)?;
        }
        Ok(())
    }

    /// Carrega l'estat JSON des del localStorage del navegador si existeix
    pub fn load_from_local_storage() -> Option<String> {
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        storage.get_item(STORAGE_KEY).ok()?
    }

    /// Esborra l'estat guardat al localStorage
    pub fn clear_local_storage() -> Result<(), wasm_bindgen::JsValue> {
        let window = web_sys::window().ok_or_else(|| wasm_bindgen::JsValue::from_str("No window available"))?;
        if let Ok(Some(storage)) = window.local_storage() {
            storage.remove_item(STORAGE_KEY)?;
        }
        Ok(())
    }
}

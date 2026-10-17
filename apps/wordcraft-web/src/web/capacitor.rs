use wasm_bindgen::JsCast as _;

const SETTINGS_KEY: &str = "wordcraft.ui";
const APP_FOLDER: &str = "WordCraft";
const RECOVERY_PATH: &str = "WordCraft/recovery.docx";

fn js_error(error: wasm_bindgen::JsValue) -> String {
    format!("{error:?}")
}

fn object(fields: &[(&str, wasm_bindgen::JsValue)]) -> Result<js_sys::Object, String> {
    let result = js_sys::Object::new();
    for (key, value) in fields {
        js_sys::Reflect::set(&result, &wasm_bindgen::JsValue::from_str(key), value).map_err(js_error)?;
    }
    Ok(result)
}

fn string(value: &str) -> wasm_bindgen::JsValue {
    wasm_bindgen::JsValue::from_str(value)
}

fn plugin(name: &str) -> Result<wasm_bindgen::JsValue, String> {
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let capacitor = js_sys::Reflect::get(&window, &string("Capacitor")).map_err(js_error)?;
    let plugins = js_sys::Reflect::get(&capacitor, &string("Plugins")).map_err(js_error)?;
    let result = js_sys::Reflect::get(&plugins, &string(name)).map_err(js_error)?;
    if result.is_undefined() || result.is_null() {
        return Err(format!("Capacitor plugin {name} is unavailable"));
    }
    Ok(result)
}

fn invoke(name: &str, method: &str, options: &js_sys::Object) -> Result<js_sys::Promise, String> {
    let target = plugin(name)?;
    let member = js_sys::Reflect::get(&target, &string(method)).map_err(js_error)?;
    let function = member.dyn_into::<js_sys::Function>().map_err(js_error)?;
    let result = function.call1(&target, options).map_err(js_error)?;
    result.dyn_into::<js_sys::Promise>().map_err(js_error)
}

async fn call(name: &str, method: &str, fields: &[(&str, wasm_bindgen::JsValue)]) -> Result<wasm_bindgen::JsValue, String> {
    let options = object(fields)?;
    let promise = invoke(name, method, &options)?;
    wasm_bindgen_futures::JsFuture::from(promise).await.map_err(js_error)
}

pub fn is_android() -> bool {
    let Some(window) = web_sys::window() else { return false };
    let Ok(capacitor) = js_sys::Reflect::get(&window, &string("Capacitor")) else { return false };
    let Ok(get_platform) = js_sys::Reflect::get(&capacitor, &string("getPlatform")) else { return false };
    let Ok(get_platform) = get_platform.dyn_into::<js_sys::Function>() else { return false };
    get_platform.call0(&capacitor).ok().and_then(|v| v.as_string()).as_deref() == Some("android")
}

pub fn device_languages() -> Vec<String> {
    let Some(window) = web_sys::window() else { return Vec::new() };
    let Ok(navigator) = js_sys::Reflect::get(&window, &string("navigator")) else { return Vec::new() };
    let mut languages = Vec::new();
    if let Ok(value) = js_sys::Reflect::get(&navigator, &string("languages"))
        && let Ok(values) = value.dyn_into::<js_sys::Array>()
    {
        languages.extend(values.iter().filter_map(|value| value.as_string()));
    }
    if let Ok(value) = js_sys::Reflect::get(&navigator, &string("language"))
        && let Some(language) = value.as_string()
        && !languages.iter().any(|existing| existing == &language)
    {
        languages.push(language);
    }
    languages
}

pub async fn load_settings(android: bool) -> Result<Option<String>, String> {
    if android {
        let result = call("Preferences", "get", &[("key", string(SETTINGS_KEY))]).await?;
        return Ok(js_sys::Reflect::get(&result, &string("value")).map_err(js_error)?.as_string());
    }
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let storage = window.local_storage().map_err(js_error)?.ok_or_else(|| "localStorage is unavailable".to_string())?;
    storage.get_item(SETTINGS_KEY).map_err(js_error)
}

pub async fn save_settings(android: bool, value: String) -> Result<(), String> {
    if android {
        call("Preferences", "set", &[("key", string(SETTINGS_KEY)), ("value", string(&value))]).await?;
        return Ok(());
    }
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let storage = window.local_storage().map_err(js_error)?.ok_or_else(|| "localStorage is unavailable".to_string())?;
    storage.set_item(SETTINGS_KEY, &value).map_err(js_error)
}

pub async fn save_documents_file(path: &str, bytes: &[u8]) -> Result<(), String> {
    let relative = path.strip_prefix("Documents/").ok_or_else(|| "invalid Documents path".to_string())?;
    if !relative.starts_with("WordCraft/") || !valid_relative_path(relative) {
        return Err("invalid Documents path".into());
    }
    let permissions = call("Filesystem", "checkPermissions", &[]).await?;
    let permission = js_sys::Reflect::get(&permissions, &string("publicStorage")).map_err(js_error)?.as_string();
    if permission.as_deref() != Some("granted") {
        let permissions = call("Filesystem", "requestPermissions", &[]).await?;
        let permission = js_sys::Reflect::get(&permissions, &string("publicStorage")).map_err(js_error)?.as_string();
        if permission.as_deref() == Some("denied") {
            return Err("Documents storage permission was denied".into());
        }
    }
    call(
        "Filesystem",
        "writeFile",
        &[
            ("path", string(relative)),
            ("data", string(&wordcraft_engine::cmd::insert::base64_encode(bytes))),
            ("directory", string("DOCUMENTS")),
            ("recursive", wasm_bindgen::JsValue::TRUE),
        ],
    )
    .await?;
    Ok(())
}

pub async fn write_private(path: &str, bytes: &[u8]) -> Result<(), String> {
    if !valid_relative_path(path) {
        return Err("invalid app-private file path".into());
    }
    call(
        "Filesystem",
        "writeFile",
        &[
            ("path", string(path)),
            ("data", string(&wordcraft_engine::cmd::insert::base64_encode(bytes))),
            ("directory", string("DATA")),
            ("recursive", wasm_bindgen::JsValue::TRUE),
        ],
    )
    .await?;
    Ok(())
}

pub async fn read_private(path: &str) -> Result<Vec<u8>, String> {
    if !valid_relative_path(path) {
        return Err("invalid app-private file path".into());
    }
    let result = call("Filesystem", "readFile", &[("path", string(path)), ("directory", string("DATA"))]).await?;
    let encoded = js_sys::Reflect::get(&result, &string("data")).map_err(js_error)?.as_string().ok_or_else(|| "invalid file data".to_string())?;
    wordcraft_engine::cmd::insert::base64_decode(&encoded).ok_or_else(|| "invalid base64 file data".to_string())
}

pub async fn remove_private(path: &str) -> Result<(), String> {
    if !valid_relative_path(path) {
        return Err("invalid app-private file path".into());
    }
    call("Filesystem", "deleteFile", &[("path", string(path)), ("directory", string("DATA"))]).await?;
    Ok(())
}

pub async fn document_uri(path: &str) -> Result<String, String> {
    let relative = path.strip_prefix("Documents/").ok_or_else(|| "invalid Documents path".to_string())?;
    if !relative.starts_with("WordCraft/") || !valid_relative_path(relative) {
        return Err("invalid Documents path".into());
    }
    let result = call("Filesystem", "getUri", &[("path", string(relative)), ("directory", string("DOCUMENTS"))]).await?;
    js_sys::Reflect::get(&result, &string("uri")).map_err(js_error)?.as_string().ok_or_else(|| "filesystem returned no share URI".into())
}

pub async fn share_file(uri: &str, name: &str) -> Result<(), String> {
    let files = js_sys::Array::new();
    files.push(&string(uri));
    call("Share", "share", &[("title", string(name)), ("text", string(name)), ("files", files.into())]).await?;
    Ok(())
}

pub fn recent_cache_path(file_name: &str) -> String {
    let safe_name: String =
        file_name.chars().take(160).map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '_' }).collect();
    let stamp = js_sys::Date::now().max(0.0) as u64;
    let nonce = (js_sys::Math::random() * u32::MAX as f64) as u32;
    format!("{APP_FOLDER}/Recent/{stamp:x}-{nonce:x}/{safe_name}")
}

pub fn recent_storage_path(recent_path: &str) -> Result<&str, String> {
    let path = recent_path.strip_prefix("capacitor-recent/").ok_or_else(|| "invalid recent file".to_string())?;
    if !valid_relative_path(path) {
        return Err("invalid recent file path".into());
    }
    Ok(path)
}

pub fn recovery_path() -> &'static str {
    RECOVERY_PATH
}

fn valid_relative_path(path: &str) -> bool {
    !path.is_empty() && !path.starts_with('/') && !path.contains('\\') && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

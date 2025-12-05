use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::window;

const API: &str = "http://localhost:3000";

#[derive(Serialize)]
struct AuthReq {
    username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    password: String,
}

#[derive(Serialize)]
struct PostReq {
    title: String,
    content: String,
}

#[derive(Deserialize, Serialize)]
struct User {
    id: i64,
    username: String,
    email: String,
}

#[derive(Deserialize, Serialize)]
struct AuthResp {
    token: String,
    user: User,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub author_username: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, Serialize)]
struct PostsResp {
    posts: Vec<Post>,
    total: i64,
}

fn storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok()?
}

fn save_token(t: &str) {
    if let Some(s) = storage() {
        let _ = s.set_item("token", t);
    }
}

fn get_token() -> Option<String> {
    storage()?.get_item("token").ok()?
}

fn clear_token() {
    if let Some(s) = storage() {
        let _ = s.remove_item("token");
    }
}

fn to_js<T: Serialize>(v: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(v).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn js_err(msg: &str) -> JsValue {
    JsValue::from_str(msg)
}

async fn do_request<T: DeserializeOwned>(req: gloo_net::http::Request) -> Result<T, JsValue> {
    let resp = req.send().await.map_err(|e| js_err(&e.to_string()))?;
    if !resp.ok() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(js_err(&txt));
    }
    resp.json().await.map_err(|e| js_err(&e.to_string()))
}

#[wasm_bindgen]
pub struct BlogApp {
    base: String,
}

#[wasm_bindgen]
impl BlogApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { base: API.into() }
    }

    pub fn is_authenticated(&self) -> bool {
        get_token().is_some()
    }

    pub fn logout(&self) {
        clear_token();
    }

    pub fn current_user(&self) -> Result<JsValue, JsValue> {
        let token = match get_token() {
            Some(t) => t,
            None => return Ok(JsValue::NULL),
        };
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Ok(JsValue::NULL);
        }
        let payload = base64_decode(parts[1]).map_err(|_| js_err("bad token"))?;
        let claims: serde_json::Value = serde_json::from_str(&payload).map_err(|_| js_err("bad token"))?;
        to_js(&claims)
    }

    pub async fn register(&self, username: String, email: String, password: String) -> Result<JsValue, JsValue> {
        let req = Request::post(&format!("{}/api/auth/register", self.base))
            .header("Content-Type", "application/json")
            .json(&AuthReq { username, email: Some(email), password })
            .map_err(|e| js_err(&e.to_string()))?;

        let auth: AuthResp = do_request(req).await?;
        save_token(&auth.token);
        to_js(&auth)
    }

    pub async fn login(&self, username: String, password: String) -> Result<JsValue, JsValue> {
        let req = Request::post(&format!("{}/api/auth/login", self.base))
            .header("Content-Type", "application/json")
            .json(&AuthReq { username, email: None, password })
            .map_err(|e| js_err(&e.to_string()))?;

        let auth: AuthResp = do_request(req).await?;
        save_token(&auth.token);
        to_js(&auth)
    }

    pub async fn load_posts(&self) -> Result<JsValue, JsValue> {
        let resp = Request::get(&format!("{}/api/posts?limit=50", self.base))
            .send()
            .await
            .map_err(|e| js_err(&e.to_string()))?;

        if !resp.ok() {
            return Err(js_err(&resp.text().await.unwrap_or_default()));
        }
        let data: PostsResp = resp.json().await.map_err(|e| js_err(&e.to_string()))?;
        to_js(&data)
    }

    pub async fn create_post(&self, title: String, content: String) -> Result<JsValue, JsValue> {
        let token = get_token().ok_or_else(|| js_err("not logged in"))?;
        let req = Request::post(&format!("{}/api/posts", self.base))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&PostReq { title, content })
            .map_err(|e| js_err(&e.to_string()))?;

        let post: Post = do_request(req).await?;
        to_js(&post)
    }

    pub async fn update_post(&self, id: i64, title: String, content: String) -> Result<JsValue, JsValue> {
        let token = get_token().ok_or_else(|| js_err("not logged in"))?;
        let req = Request::put(&format!("{}/api/posts/{}", self.base, id))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&PostReq { title, content })
            .map_err(|e| js_err(&e.to_string()))?;

        let post: Post = do_request(req).await?;
        to_js(&post)
    }

    pub async fn delete_post(&self, id: i64) -> Result<JsValue, JsValue> {
        let token = get_token().ok_or_else(|| js_err("not logged in"))?;
        let resp = Request::delete(&format!("{}/api/posts/{}", self.base, id))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| js_err(&e.to_string()))?;

        if resp.ok() || resp.status() == 204 {
            Ok(JsValue::TRUE)
        } else {
            Err(js_err(&resp.text().await.unwrap_or_default()))
        }
    }
}

fn base64_decode(input: &str) -> Result<String, ()> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let input = input.replace('-', "+").replace('_', "/");
    let pad = (4 - input.len() % 4) % 4;
    let padded = format!("{}{}", input, "=".repeat(pad));

    let mut out = Vec::new();
    for chunk in padded.as_bytes().chunks(4) {
        if chunk.len() != 4 { break; }
        let idx: Vec<u8> = chunk.iter().map(|&b| {
            if b == b'=' { 0 } else { CHARS.iter().position(|&c| c == b).unwrap_or(0) as u8 }
        }).collect();

        out.push((idx[0] << 2) | (idx[1] >> 4));
        if chunk[2] != b'=' { out.push((idx[1] << 4) | (idx[2] >> 2)); }
        if chunk[3] != b'=' { out.push((idx[2] << 6) | idx[3]); }
    }
    String::from_utf8(out).map_err(|_| ())
}

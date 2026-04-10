use crate::trace;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn client_id() -> String {
    std::env::var("GOOGLE_OAUTH_CLIENT_ID")
        .expect("GOOGLE_OAUTH_CLIENT_ID must be set")
}
fn client_secret() -> String {
    std::env::var("GOOGLE_OAUTH_CLIENT_SECRET")
        .expect("GOOGLE_OAUTH_CLIENT_SECRET must be set")
}
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const SCOPE: &str = "https://mail.google.com/ https://www.googleapis.com/auth/calendar openid email";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub email: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
}

pub async fn start_flow() -> Result<OAuthTokens, String> {
    trace::trace("OAUTH", "Starting OAuth flow");

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("ローカルサーバー起動失敗: {e}"))?;
    let port = listener.local_addr().map_err(|e| format!("{e}"))?.port();
    let redirect_uri = format!("http://127.0.0.1:{port}");
    trace::trace("OAUTH", &format!("Listening on port {port}"));

    let cid = client_id();
    let url = reqwest::Url::parse_with_params(AUTH_URL, &[
        ("client_id", cid.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("response_type", "code"),
        ("scope", SCOPE),
        ("access_type", "offline"),
        ("prompt", "consent"),
    ])
    .map_err(|e| format!("{e}"))?;

    std::process::Command::new("open")
        .arg(url.as_str())
        .spawn()
        .map_err(|e| format!("ブラウザ起動失敗: {e}"))?;
    trace::trace("OAUTH", "Browser opened");

    let (mut stream, _) = listener
        .accept()
        .await
        .map_err(|e| format!("コールバック受信失敗: {e}"))?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.map_err(|e| format!("{e}"))?;
    let request = String::from_utf8_lossy(&buf[..n]);
    trace::trace("OAUTH", "Received callback");

    let code = request
        .lines()
        .next()
        .and_then(|line| line.split('?').nth(1))
        .and_then(|query_and_rest| query_and_rest.split_whitespace().next())
        .and_then(|query| query.split('&').find_map(|p| p.strip_prefix("code=")))
        .ok_or("認証コードが見つかりません".to_string())?;

    let html = "<html><body style='font-family:sans-serif;text-align:center;padding:40px;background:#1e1e2e;color:#cdd6f4'>\
        <h2>認証完了</h2><p>SmartAMに戻ってください。このタブは閉じて構いません。</p></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{html}"
    );
    stream.write_all(response.as_bytes()).await.ok();

    trace::trace("OAUTH", "Exchanging code for tokens");
    exchange_code(code, &redirect_uri).await
}

async fn exchange_code(code: &str, redirect_uri: &str) -> Result<OAuthTokens, String> {
    let cid = client_id();
    let csec = client_secret();
    let client = reqwest::Client::new();
    let resp = client
        .post(TOKEN_URL)
        .form(&[
            ("code", code),
            ("client_id", cid.as_str()),
            ("client_secret", csec.as_str()),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| format!("トークン交換失敗: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("トークン交換エラー: {body}"));
    }

    let data: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("レスポンス解析失敗: {e}"))?;

    let expires_at = chrono::Utc::now().timestamp() + data.expires_in;
    let email = fetch_email(&data.access_token).await?;
    trace::trace("OAUTH", &format!("Token exchange successful, email: {email}"));

    Ok(OAuthTokens {
        access_token: data.access_token,
        refresh_token: data.refresh_token.unwrap_or_default(),
        expires_at,
        email,
    })
}

async fn fetch_email(access_token: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    struct UserInfo { email: String }

    let client = reqwest::Client::new();
    let resp = client
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("ユーザー情報取得失敗: {e}"))?;

    let info: UserInfo = resp
        .json()
        .await
        .map_err(|e| format!("ユーザー情報解析失敗: {e}"))?;
    Ok(info.email)
}

pub async fn refresh(refresh_token: &str) -> Result<OAuthTokens, String> {
    trace::trace("OAUTH", "Refreshing access token");
    let cid = client_id();
    let csec = client_secret();
    let client = reqwest::Client::new();
    let resp = client
        .post(TOKEN_URL)
        .form(&[
            ("client_id", cid.as_str()),
            ("client_secret", csec.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| format!("トークン更新失敗: {e}"))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("トークン更新エラー: {body}"));
    }

    let data: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("レスポンス解析失敗: {e}"))?;

    let expires_at = chrono::Utc::now().timestamp() + data.expires_in;
    trace::trace("OAUTH", "Token refresh successful");

    Ok(OAuthTokens {
        access_token: data.access_token,
        refresh_token: refresh_token.to_string(),
        expires_at,
        email: String::new(),
    })
}

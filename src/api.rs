use std::io::Cursor;

use image::Luma;
use qrcode::QrCode;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
struct AuthRequest<'a> {
    grant_type: &'a str,
    username: &'a str,
    password: &'a str,
    scope: &'a str,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AuthResponse {
    access_token: String,
    expires_in: i32,
    refresh_token: String,
    scope: String,
    token_type: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct GetQRCodeResponse {
    #[serde(rename = "ExpiresAt")]
    expires_at: String,
    #[serde(rename = "ExpiresIn")]
    expires_in: String,
    #[serde(rename = "QrCode")]
    qr_code: String,
    #[serde(rename = "RefreshAt")]
    refresh_at: String,
    #[serde(rename = "RefreshIn")]
    refresh_in: String,
}

impl<'a> From<&'a LoginCredentials> for AuthRequest<'a> {
    fn from(value: &'a LoginCredentials) -> Self {
        Self {
            grant_type: "password",
            username: value.email.as_str(),
            password: value.password.as_str(),
            scope: "pgcapi offline_access",
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum APIError {
    #[error("Failed to call PureGym API.")]
    RequestError(#[source] reqwest::Error),
    #[error("Failed to process response.")]
    ResponseError(#[source] reqwest::Error),
    #[error("Failed to create QR code.")]
    QRProcessingError(#[source] anyhow::Error),
    #[error("Something unexpected went wrong!")]
    InternalError(#[source] anyhow::Error),
}

fn fetch_auth_token(credentials: &LoginCredentials) -> Result<String, APIError> {
    let auth_url = "https://auth.puregym.com/connect/token";
    let client = Client::new();
    let body = serde_urlencoded::to_string(&AuthRequest::from(credentials))
        .map_err(|e| APIError::InternalError(e.into()))?;
    let response: AuthResponse = client
        .post(auth_url)
        .body(body)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(AUTHORIZATION, "Basic cm8uY2xpZW50Og==")
        .send()
        .map_err(APIError::RequestError)?
        .json()
        .map_err(APIError::ResponseError)?;
    Ok(response.access_token)
}

pub fn fetch_qr_content(credentials: &LoginCredentials) -> Result<String, APIError> {
    let token = fetch_auth_token(credentials)?;
    let client = Client::new();
    let get_qr_url = "https://capi.puregym.com/api/v2/member/qrcode";
    let response: GetQRCodeResponse = client
        .get(get_qr_url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .map_err(APIError::RequestError)?
        .json()
        .map_err(APIError::ResponseError)?;
    Ok(response.qr_code)
}

pub fn generate_qr(credentials: &LoginCredentials) -> Result<Vec<u8>, APIError> {
    let qr_content = fetch_qr_content(credentials)?;
    let qr_code = QrCode::new(qr_content).map_err(|e| APIError::QRProcessingError(e.into()))?;
    let image = qr_code.render::<Luma<u8>>().build();
    let mut bytes = vec![];
    image
        .write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)
        .map_err(|e| APIError::QRProcessingError(e.into()))?;
    Ok(bytes)
}

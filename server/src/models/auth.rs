use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SendVerificationCodeRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct VerifyCodeRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6, max = 6))]
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i32,
    pub email: String,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub ethereum_address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VerificationCode {
    pub email: String,
    pub code: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendCodeResponse {
    pub message: String,
    pub code: Option<String>, // Only included when SMTP is not configured
}

// Web3/MetaMask related models
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct MetaMaskChallengeRequest {
    #[validate(length(min = 42, max = 42))]
    pub address: String, // Ethereum address
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaMaskChallengeResponse {
    pub message: String,
    pub nonce: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct MetaMaskVerifyRequest {
    #[validate(length(min = 42, max = 42))]
    pub address: String,
    pub signature: String,
    pub message: String,
    pub nonce: String,
}

#[derive(Debug, Clone)]
pub struct Web3Challenge {
    pub address: String,
    pub message: String,
    pub nonce: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
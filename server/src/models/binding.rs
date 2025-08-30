use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserAccountBinding {
    pub id: i32,
    pub primary_user_id: i32,
    pub secondary_user_id: Option<i32>,
    pub binding_type: String,
    pub email: Option<String>,
    pub ethereum_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BindEmailToWalletRequest {
    #[validate(length(min = 42, max = 42))]
    pub ethereum_address: String,
    #[validate(length(min = 6, max = 6))]
    pub verification_code: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BindWalletToEmailRequest {
    #[validate(email)]
    pub email: String,
    pub signature: String,
    pub message: String,
    pub nonce: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountBindingResponse {
    pub success: bool,
    pub message: String,
    pub binding: Option<UserAccountBinding>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBindingsResponse {
    pub user_id: i32,
    pub email: Option<String>,
    pub ethereum_address: Option<String>,
    pub has_email_binding: bool,
    pub has_wallet_binding: bool,
    pub bindings: Vec<UserAccountBinding>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UnbindAccountRequest {
    pub binding_type: String, // "email" or "wallet"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnbindAccountResponse {
    pub success: bool,
    pub message: String,
}

// Challenge request for wallet binding
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct WalletBindingChallengeRequest {
    #[validate(length(min = 42, max = 42))]
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletBindingChallengeResponse {
    pub message: String,
    pub nonce: String,
}
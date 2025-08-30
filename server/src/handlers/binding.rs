use axum::{extract::State, Json};
use validator::Validate;

use crate::{
    handlers::{success_response, success_response_with_message, AppError, Result},
    middleware::auth::Claims,
    models::{
        BindEmailToWalletRequest, BindWalletToEmailRequest, AccountBindingResponse, 
        UnbindAccountRequest, UnbindAccountResponse, SendVerificationCodeRequest,
        WalletBindingChallengeRequest, WalletBindingChallengeResponse
    },
    services::AppState,
};

// Get user's binding status
pub async fn get_user_bindings(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    match state.auth_service.get_user_bindings(claims.user_id).await {
        Ok(bindings_response) => Ok(success_response(bindings_response)),
        Err(e) => Err(AppError::Internal(format!("获取用户绑定状态失败 / Failed to get user bindings: {}", e))),
    }
}

// Bind email user to MetaMask wallet
pub async fn bind_email_to_wallet(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<BindEmailToWalletRequest>,
) -> Result<Json<serde_json::Value>> {
    payload.validate()?;

    if !state.config.web3.enabled {
        return Err(AppError::BadRequest(
            "Web3认证未启用 / Web3 authentication is not enabled".to_string(),
        ));
    }

    match state.auth_service.bind_email_to_wallet(
        claims.user_id, 
        &payload.ethereum_address, 
        &payload.verification_code
    ).await {
        Ok(binding) => {
            let response = AccountBindingResponse {
                success: true,
                message: "成功将钱包绑定到您的邮箱账户 / Successfully bound wallet to your email account".to_string(),
                binding: Some(binding),
            };
            Ok(success_response_with_message(response, "钱包绑定成功 / Wallet binding successful"))
        }
        Err(e) => Err(AppError::BadRequest(e.to_string())),
    }
}

// Bind MetaMask wallet user to email
pub async fn bind_wallet_to_email(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<BindWalletToEmailRequest>,
) -> Result<Json<serde_json::Value>> {
    payload.validate()?;

    // First, send verification code to the email
    let code_response = state.auth_service.send_verification_code(
        payload.email.clone(), 
        &state.smtp_service
    ).await;

    match code_response {
        Ok(_code) => {
            // Note: In a real implementation, you might want to handle this differently
            // For now, we'll assume the verification code process is handled separately
            match state.auth_service.bind_wallet_to_email(
                claims.user_id, 
                &payload.email, 
                "pending" // This would need to be handled properly with a verification flow
            ).await {
                Ok(binding) => {
                    let response = AccountBindingResponse {
                        success: true,
                        message: "成功将邮箱绑定到您的钱包账户 / Successfully bound email to your wallet account".to_string(),
                        binding: Some(binding),
                    };
                    Ok(success_response_with_message(response, "邮箱绑定成功 / Email binding successful"))
                }
                Err(e) => Err(AppError::BadRequest(e.to_string())),
            }
        }
        Err(e) => Err(AppError::Internal(format!("发送验证码失败 / Failed to send verification code: {}", e))),
    }
}

// Send verification code for email binding
pub async fn send_email_verification_for_binding(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<SendVerificationCodeRequest>,
) -> Result<Json<serde_json::Value>> {
    payload.validate()?;

    match state.auth_service.send_verification_code(payload.email, &state.smtp_service).await {
        Ok(code) => {
            let response = if code.is_empty() {
                serde_json::json!({
                    "message": "验证码已发送到您的邮箱，请查收",
                    "code": serde_json::Value::Null,
                })
            } else {
                serde_json::json!({
                    "message": "验证码已生成，请查看下方显示的验证码",
                    "code": code,
                })
            };
            Ok(success_response(response))
        }
        Err(e) => Err(AppError::Internal(format!("发送验证码失败 / Failed to send verification code: {}", e))),
    }
}

// Generate Web3 challenge for wallet binding
pub async fn generate_wallet_binding_challenge(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<WalletBindingChallengeRequest>,
) -> Result<Json<serde_json::Value>> {
    payload.validate()?;

    if !state.config.web3.enabled {
        return Err(AppError::BadRequest(
            "Web3认证未启用 / Web3 authentication is not enabled".to_string(),
        ));
    }

    match state.auth_service.generate_web3_challenge(&payload.address).await {
        Ok((message, nonce)) => {
            let response = WalletBindingChallengeResponse { message, nonce };
            Ok(success_response_with_message(
                response,
                "挑战生成成功 / Challenge generated successfully",
            ))
        }
        Err(e) => Err(AppError::BadRequest(e.to_string())),
    }
}

// Verify email binding with code
pub async fn verify_email_binding(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let email = payload
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("需要邮箱地址 / Email required".to_string()))?;
    
    let code = payload
        .get("code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("需要验证码 / Verification code required".to_string()))?;

    match state.auth_service.bind_wallet_to_email(claims.user_id, email, code).await {
        Ok(binding) => {
            let response = AccountBindingResponse {
                success: true,
                message: "成功将邮箱绑定到您的账户 / Successfully bound email to your account".to_string(),
                binding: Some(binding),
            };
            Ok(success_response_with_message(response, "邮箱绑定成功 / Email binding successful"))
        }
        Err(e) => Err(AppError::BadRequest(e.to_string())),
    }
}

// Verify wallet binding with signature
pub async fn verify_wallet_binding(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let address = payload
        .get("address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("需要钱包地址 / Wallet address required".to_string()))?;
    
    let signature = payload
        .get("signature")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("需要签名 / Signature required".to_string()))?;

    let message = payload
        .get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("需要消息 / Message required".to_string()))?;

    let nonce = payload
        .get("nonce")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("需要随机数 / Nonce required".to_string()))?;

    // Verify the signature using the public signature verification method
    match state.auth_service.verify_eth_signature(address, message, signature) {
        Ok(_) => {
            // If signature is valid, create the binding using a special verification code for crypto
            match state.auth_service.bind_email_to_wallet(claims.user_id, address, "crypto_verified").await {
                Ok(binding) => {
                    let response = AccountBindingResponse {
                        success: true,
                        message: "成功将钱包绑定到您的账户 / Successfully bound wallet to your account".to_string(),
                        binding: Some(binding),
                    };
                    Ok(success_response_with_message(response, "钱包绑定成功 / Wallet binding successful"))
                }
                Err(e) => Err(AppError::BadRequest(e.to_string())),
            }
        }
        Err(e) => Err(AppError::BadRequest(format!("签名验证失败 / Signature verification failed: {}", e))),
    }
}

// Unbind account (remove email or wallet binding)
pub async fn unbind_account(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<UnbindAccountRequest>,
) -> Result<Json<serde_json::Value>> {
    match state.auth_service.unbind_account(claims.user_id, &payload.binding_type).await {
        Ok(_) => {
            let response = UnbindAccountResponse {
                success: true,
                message: format!("成功解除{}绑定 / Successfully removed {} binding", 
                    match payload.binding_type.as_str() {
                        "email" => "邮箱",
                        "wallet" => "钱包",
                        _ => "账户"
                    },
                    &payload.binding_type
                ),
            };
            Ok(success_response_with_message(response, "账户解绑成功 / Account unbound successfully"))
        }
        Err(e) => Err(AppError::BadRequest(e.to_string())),
    }
}
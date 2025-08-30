use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sqlx::PgPool;
use ipnetwork::IpNetwork;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::{
    models::{LoginResponse, TokenClaims, User, UserResponse, VerificationCode, AuthResponse, UserInfo, Web3Challenge, UserAccountBinding, UserBindingsResponse},
    utils::config::Config,
};

pub struct AuthService {
    db_pool: PgPool,
    config: Arc<Config>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    verification_codes: Arc<RwLock<HashMap<String, VerificationCode>>>,
    web3_challenges: Arc<RwLock<HashMap<String, Web3Challenge>>>,
}

impl AuthService {
    pub fn new(db_pool: PgPool, config: Arc<Config>) -> Self {
        let encoding_key = EncodingKey::from_secret(config.jwt.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt.secret.as_bytes());

        Self {
            db_pool,
            config,
            encoding_key,
            decoding_key,
            verification_codes: Arc::new(RwLock::new(HashMap::new())),
            web3_challenges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn user_exists(&self, username: &str, email: &str) -> sqlx::Result<bool> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE username = $1 OR email = $2"
        )
        .bind(username)
        .bind(email)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(count > 0)
    }

    pub async fn register_user(
        &self,
        username: String,
        email: String,
        password: String,
        display_name: Option<String>,
    ) -> sqlx::Result<UserResponse> {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|_| sqlx::Error::Protocol("Password hashing failed".to_string()))?;

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash, display_name)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(&username)
        .bind(&email)
        .bind(&password_hash)
        .bind(&display_name)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(user.into())
    }

    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> sqlx::Result<Option<LoginResponse>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = $1 AND is_active = true"
        )
        .bind(username)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(user) = user {
            if bcrypt::verify(password, &user.password_hash)
                .map_err(|_| sqlx::Error::Protocol("Password verification failed".to_string()))?
            {
                let (access_token, refresh_token) = self.generate_tokens(&user)?;
                
                return Ok(Some(LoginResponse {
                    access_token,
                    refresh_token,
                    expires_in: self.config.jwt.access_token_expires_in,
                    user: user.into(),
                }));
            }
        }

        Ok(None)
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> sqlx::Result<Option<LoginResponse>> {
        let claims = self.verify_token(refresh_token)
            .map_err(|_| sqlx::Error::Protocol("Invalid token".to_string()))?;
        
        let user_id = claims.sub.parse::<i32>()
            .map_err(|_| sqlx::Error::Protocol("Invalid user ID in token".to_string()))?;
        
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(user) = user {
            let (access_token, refresh_token) = self.generate_tokens(&user)?;
            
            return Ok(Some(LoginResponse {
                access_token,
                refresh_token,
                expires_in: self.config.jwt.access_token_expires_in,
                user: user.into(),
            }));
        }

        Ok(None)
    }

    pub fn verify_token(&self, token: &str) -> Result<TokenClaims, jsonwebtoken::errors::Error> {
        let validation = Validation::default();
        let token_data = decode::<TokenClaims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }

    fn generate_tokens(&self, user: &User) -> sqlx::Result<(String, String)> {
        let now = Utc::now();
        let access_exp = (now + Duration::seconds(self.config.jwt.access_token_expires_in)).timestamp() as usize;
        let refresh_exp = (now + Duration::seconds(self.config.jwt.refresh_token_expires_in)).timestamp() as usize;

        let access_claims = TokenClaims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            exp: access_exp,
            iat: now.timestamp() as usize,
        };

        let refresh_claims = TokenClaims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            exp: refresh_exp,
            iat: now.timestamp() as usize,
        };

        let access_token = encode(&Header::default(), &access_claims, &self.encoding_key)
            .map_err(|_| sqlx::Error::Protocol("Token generation failed".to_string()))?;

        let refresh_token = encode(&Header::default(), &refresh_claims, &self.encoding_key)
            .map_err(|_| sqlx::Error::Protocol("Token generation failed".to_string()))?;

        Ok((access_token, refresh_token))
    }

    // Email verification code methods
    pub async fn send_verification_code(&self, email: String, smtp_service: &crate::services::smtp::SmtpService) -> anyhow::Result<String> {
        // Generate 6-digit code
        let code = format!("{:06}", fastrand::u32(100000..1000000));
        
        // Store code with 10 minute expiration
        let verification_code = VerificationCode {
            email: email.clone(),
            code: code.clone(),
            expires_at: Utc::now() + Duration::minutes(10),
        };

        // Clean up expired codes and store new one
        self.cleanup_expired_codes().await;
        {
            let mut codes = self.verification_codes.write().await;
            codes.insert(email.clone(), verification_code);
        }

        // Try to send email via SMTP if configured
        match smtp_service.send_verification_code(&email, &code).await {
            Ok(true) => {
                // Email sent successfully, return empty string to indicate no display needed
                tracing::info!("Verification code sent via email to {}", email);
                Ok("".to_string())
            },
            Ok(false) => {
                // SMTP not configured or failed, return code for display
                tracing::info!("SMTP not configured, returning code for display: {}", code);
                Ok(code)
            },
            Err(e) => {
                // SMTP error, fallback to display mode
                tracing::warn!("SMTP error, falling back to display mode: {}", e);
                Ok(code)
            }
        }
    }

    pub async fn verify_code_and_auth(&self, email: String, code: String, ip_address: Option<std::net::IpAddr>, user_agent: Option<&str>) -> anyhow::Result<AuthResponse> {
        // Check verification code
        {
            let codes = self.verification_codes.read().await;
            if let Some(stored_code) = codes.get(&email) {
                if stored_code.expires_at < Utc::now() {
                    return Err(anyhow::anyhow!("验证码已过期"));
                }
                if stored_code.code != code {
                    return Err(anyhow::anyhow!("验证码错误"));
                }
            } else {
                return Err(anyhow::anyhow!("验证码不存在或已过期"));
            }
        }

        // Remove used code
        {
            let mut codes = self.verification_codes.write().await;
            codes.remove(&email);
        }

        // Find or create user, checking for bindings
        let user = match self.find_user_by_binding(Some(&email), None).await? {
            Some(bound_user) => bound_user,
            None => self.find_or_create_user_by_email(&email).await?,
        };

        // Generate JWT token
        let (access_token, _) = self.generate_tokens(&user)
            .map_err(|e| anyhow::anyhow!("Token generation failed: {}", e))?;

        // Record login activity
        if let Err(e) = self.record_login_activity(user.id, &user.email, ip_address, user_agent, true, None).await {
            tracing::warn!("Failed to record login activity: {}", e);
        }

        Ok(AuthResponse {
            token: access_token,
            user: UserInfo {
                id: user.id,
                email: user.email,
                display_name: user.display_name,
                role: user.role,
                ethereum_address: user.ethereum_address,
            },
        })
    }

    async fn find_or_create_user_by_email(&self, email: &str) -> anyhow::Result<User> {
        // Try to find existing user
        if let Ok(user) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(&self.db_pool)
            .await
        {
            return Ok(user);
        }

        // Check if this is the first user
        let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&self.db_pool)
            .await?;

        let is_first_user = user_count == 0;

        // Create new user
        let username = email.split('@').next().unwrap_or("user").to_string();
        let display_name = if is_first_user {
            format!("管理员 {}", &username[..std::cmp::min(username.len(), 6)])
        } else {
            format!("用户{}", &username[..std::cmp::min(username.len(), 6)])
        };

        let role = if is_first_user { "admin" } else { "user" };

        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, password_hash, display_name, role, is_active, is_verified, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, true, true, NOW(), NOW())
             RETURNING *"
        )
        .bind(&username)
        .bind(email)
        .bind("") // Empty password for email-only auth
        .bind(&display_name)
        .bind(role)
        .fetch_one(&self.db_pool)
        .await?;

        if is_first_user {
            tracing::info!("First user registered as admin: {}", email);
        } else {
            tracing::info!("New user registered: {}", email);
        }

        Ok(user)
    }

    async fn cleanup_expired_codes(&self) {
        let now = Utc::now();
        let mut codes = self.verification_codes.write().await;
        codes.retain(|_, code| code.expires_at > now);
    }

    // Record login activity
    async fn record_login_activity(
        &self,
        user_id: i32,
        email: &str,
        ip_address: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
        is_successful: bool,
        failure_reason: Option<&str>,
    ) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_login_activities 
            (user_id, email, ip_address, user_agent, login_time, login_method, is_successful, failure_reason)
            VALUES ($1, $2, $3, $4, NOW(), 'email_verification', $5, $6)
            "#,
        )
        .bind(user_id)
        .bind(email)
        .bind(ip_address.map(|ip| IpNetwork::from(ip)))
        .bind(user_agent)
        .bind(is_successful)
        .bind(failure_reason)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    // Public getter for database pool (needed for auth middleware)
    pub fn get_db_pool(&self) -> &PgPool {
        &self.db_pool
    }

    // Web3/MetaMask authentication methods
    pub async fn generate_web3_challenge(&self, address: &str) -> anyhow::Result<(String, String)> {
        if !self.config.web3.enabled {
            return Err(anyhow::anyhow!("Web3 authentication is not enabled"));
        }

        // Validate Ethereum address format
        if !address.starts_with("0x") || address.len() != 42 {
            return Err(anyhow::anyhow!("Invalid Ethereum address format"));
        }

        let address = address.to_lowercase();

        // Generate nonce
        let nonce = uuid::Uuid::new_v4().to_string();
        
        // Create challenge message following EIP-4361 standard (Sign-In with Ethereum)
        let domain = "geektools.dev"; // You might want to make this configurable
        let issued_at = chrono::Utc::now().to_rfc3339();
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(self.config.web3.challenge_expires_in as i64);
        
        let message = format!(
            "GeekTools Plugin Marketplace wants you to sign in with your Ethereum account:\n{}\n\nSign in to access the GeekTools Plugin Marketplace.\n\nURI: https://{}\nVersion: 1\nChain ID: 1\nNonce: {}\nIssued At: {}\nExpiration Time: {}",
            address, domain, nonce, issued_at, expires_at.to_rfc3339()
        );

        // Store challenge
        let challenge = Web3Challenge {
            address: address.clone(),
            message: message.clone(),
            nonce: nonce.clone(),
            expires_at,
        };

        // Clean up expired challenges and store new one
        self.cleanup_expired_web3_challenges().await;
        {
            let mut challenges = self.web3_challenges.write().await;
            challenges.insert(address.clone(), challenge);
        }

        tracing::info!("Generated Web3 challenge for address: {}", address);

        Ok((message, nonce))
    }

    pub async fn verify_web3_signature(
        &self, 
        address: &str, 
        signature: &str, 
        message: &str, 
        nonce: &str,
        ip_address: Option<std::net::IpAddr>, 
        user_agent: Option<&str>
    ) -> anyhow::Result<AuthResponse> {
        if !self.config.web3.enabled {
            return Err(anyhow::anyhow!("Web3 authentication is not enabled"));
        }

        let address = address.to_lowercase();

        // Retrieve and validate challenge
        let stored_challenge = {
            let challenges = self.web3_challenges.read().await;
            challenges.get(&address).cloned()
        };

        let challenge = stored_challenge
            .ok_or_else(|| anyhow::anyhow!("Challenge not found or expired"))?;

        // Validate challenge hasn't expired
        if challenge.expires_at < chrono::Utc::now() {
            return Err(anyhow::anyhow!("Challenge has expired"));
        }

        // Validate message and nonce match
        if challenge.message != message || challenge.nonce != nonce {
            return Err(anyhow::anyhow!("Challenge validation failed"));
        }

        // Verify signature
        self.verify_eth_signature(&address, message, signature)
            .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))?;

        // Remove used challenge
        {
            let mut challenges = self.web3_challenges.write().await;
            challenges.remove(&address);
        }

        // Find or create user by Ethereum address, checking for bindings
        let user = match self.find_user_by_binding(None, Some(&address)).await? {
            Some(bound_user) => bound_user,
            None => self.find_or_create_user_by_eth_address(&address).await?,
        };

        // Generate JWT token
        let (access_token, _) = self.generate_tokens(&user)
            .map_err(|e| anyhow::anyhow!("Token generation failed: {}", e))?;

        // Record login activity
        if let Err(e) = self.record_web3_login_activity(user.id, &address, ip_address, user_agent, true, None).await {
            tracing::warn!("Failed to record Web3 login activity: {}", e);
        }

        tracing::info!("Successful Web3 authentication for address: {}", address);

        Ok(AuthResponse {
            token: access_token,
            user: UserInfo {
                id: user.id,
                email: user.email,
                display_name: user.display_name,
                role: user.role,
                ethereum_address: user.ethereum_address,
            },
        })
    }

    pub fn verify_eth_signature(&self, address: &str, message: &str, signature_hex: &str) -> anyhow::Result<()> {
        use ethers::core::utils::keccak256;
        use ethers::utils::hex;
        use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

        // Remove 0x prefix if present
        let signature_hex = signature_hex.strip_prefix("0x").unwrap_or(signature_hex);
        
        if signature_hex.len() != 130 {
            return Err(anyhow::anyhow!("Invalid signature length"));
        }

        // Decode signature
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|_| anyhow::anyhow!("Invalid signature hex"))?;

        if signature_bytes.len() != 65 {
            return Err(anyhow::anyhow!("Invalid signature bytes length"));
        }

        // Split signature into r, s, v components
        let r = &signature_bytes[0..32];
        let s = &signature_bytes[32..64];
        let v = signature_bytes[64];

        // Create signature from r and s
        let mut sig_bytes = [0u8; 64];
        sig_bytes[..32].copy_from_slice(r);
        sig_bytes[32..].copy_from_slice(s);
        
        let signature = Signature::from_bytes(&sig_bytes.into())
            .map_err(|_| anyhow::anyhow!("Invalid signature format"))?;

        // Recovery ID (v - 27 for legacy, or v for EIP-155)
        let recovery_id = if v >= 27 {
            RecoveryId::try_from(v - 27)?
        } else {
            RecoveryId::try_from(v)?
        };

        // Create Ethereum signed message hash
        let eth_message = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
        let message_hash = keccak256(eth_message.as_bytes());

        // Recover public key from signature
        let verifying_key = VerifyingKey::recover_from_prehash(&message_hash, &signature, recovery_id)
            .map_err(|_| anyhow::anyhow!("Failed to recover public key"))?;

        // Convert public key to Ethereum address
        let public_key_bytes = verifying_key.to_encoded_point(false);
        let public_key_hash = keccak256(&public_key_bytes.as_bytes()[1..]);
        let recovered_address = format!("0x{}", hex::encode(&public_key_hash[12..]));

        // Compare addresses
        if recovered_address.to_lowercase() != address.to_lowercase() {
            return Err(anyhow::anyhow!("Signature verification failed: address mismatch"));
        }

        Ok(())
    }

    async fn find_or_create_user_by_eth_address(&self, address: &str) -> anyhow::Result<User> {
        // Try to find existing user by ethereum address
        if let Ok(user) = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE ethereum_address = $1 AND is_active = true"
        )
        .bind(address)
        .fetch_one(&self.db_pool)
        .await
        {
            tracing::info!("Found existing user with wallet address: {}", address);
            return Ok(user);
        }

        // Check if this is the first user in the system
        let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&self.db_pool)
            .await?;

        let is_first_user = user_count == 0;

        // If this is the first user, allow creating admin account with wallet
        // Otherwise, encourage binding to existing accounts
        if !is_first_user {
            // For subsequent users, we still allow creating new accounts
            // but they should consider binding to existing email accounts instead
            tracing::info!("Wallet address {} not bound to any existing user, creating new account", address);
        }

        // Try to create a unique username
        let mut username_attempts = 0;
        let base_username = format!("user_{}", &address[2..8]); // Use first 6 chars after 0x
        let mut username = base_username.clone();
        
        // Ensure username is unique
        loop {
            let existing_user = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE username = $1"
            )
            .bind(&username)
            .fetch_one(&self.db_pool)
            .await?;
            
            if existing_user == 0 {
                break;
            }
            
            username_attempts += 1;
            username = format!("{}_{}", base_username, username_attempts);
            
            if username_attempts > 100 {
                return Err(anyhow::anyhow!("Unable to generate unique username"));
            }
        }

        let display_name = if is_first_user {
            format!("Admin {}", &username)
        } else {
            format!("Web3User {}", &username)
        };

        let role = if is_first_user { "admin" } else { "user" };

        // Create user with ethereum address
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, password_hash, display_name, role, ethereum_address, is_active, is_verified, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, true, true, NOW(), NOW())
             RETURNING *"
        )
        .bind(&username)
        .bind("") // Empty email for Web3-only auth
        .bind("") // Empty password for Web3-only auth  
        .bind(&display_name)
        .bind(role)
        .bind(address)
        .fetch_one(&self.db_pool)
        .await?;

        if is_first_user {
            tracing::info!("First Web3 user registered as admin: {}", address);
        } else {
            tracing::info!("New Web3 user registered: {}", address);
        }

        Ok(user)
    }

    async fn cleanup_expired_web3_challenges(&self) {
        let now = chrono::Utc::now();
        let mut challenges = self.web3_challenges.write().await;
        challenges.retain(|_, challenge| challenge.expires_at > now);
    }

    async fn record_web3_login_activity(
        &self,
        user_id: i32,
        address: &str,
        ip_address: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
        is_successful: bool,
        failure_reason: Option<&str>,
    ) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_login_activities 
            (user_id, email, ip_address, user_agent, login_time, login_method, is_successful, failure_reason)
            VALUES ($1, $2, $3, $4, NOW(), 'metamask_web3', $5, $6)
            "#,
        )
        .bind(user_id)
        .bind(address) // Store ethereum address in email field for Web3 logins
        .bind(ip_address.map(|ip| ipnetwork::IpNetwork::from(ip)))
        .bind(user_agent)
        .bind(is_successful)
        .bind(failure_reason)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    // Account binding methods
    pub async fn bind_email_to_wallet(&self, user_id: i32, ethereum_address: &str, verification_code: &str) -> anyhow::Result<UserAccountBinding> {
        // Special case: if verification_code is "crypto_verified", skip challenge validation
        if verification_code != "crypto_verified" {
            // Validate verification code using stored challenges
            let stored_challenge = {
                let challenges = self.web3_challenges.read().await;
                challenges.get(&ethereum_address.to_lowercase()).cloned()
            };

            if stored_challenge.is_none() {
                return Err(anyhow::anyhow!("未找到钱包地址的有效挑战 / No active challenge found for wallet address"));
            }
        }

        // Get user information
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.db_pool)
            .await?;

        // Check if this wallet is already bound to another user
        let existing_wallet_user = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT id FROM users WHERE ethereum_address = $1 AND id != $2"
        )
        .bind(ethereum_address)
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?;

        if existing_wallet_user.is_some() {
            return Err(anyhow::anyhow!("此钱包地址已绑定到其他账户 / This wallet address is already bound to another account"));
        }

        // Check if user already has a wallet binding
        let existing_binding = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT id FROM user_account_bindings WHERE primary_user_id = $1 AND binding_type = 'email_to_wallet' AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?;

        if existing_binding.is_some() {
            return Err(anyhow::anyhow!("用户已有钱包绑定 / User already has a wallet binding"));
        }

        // Update user's ethereum_address
        sqlx::query("UPDATE users SET ethereum_address = $1, updated_at = NOW() WHERE id = $2")
            .bind(ethereum_address)
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;

        // Create binding record
        let binding = sqlx::query_as::<_, UserAccountBinding>(
            r#"
            INSERT INTO user_account_bindings (primary_user_id, binding_type, ethereum_address, is_active)
            VALUES ($1, 'email_to_wallet', $2, true)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(ethereum_address)
        .fetch_one(&self.db_pool)
        .await?;

        // Clean up the challenge (only if it wasn't crypto_verified)
        if verification_code != "crypto_verified" {
            let mut challenges = self.web3_challenges.write().await;
            challenges.remove(&ethereum_address.to_lowercase());
        }

        tracing::info!("Successfully bound wallet {} to user {}", ethereum_address, user_id);
        Ok(binding)
    }

    pub async fn bind_wallet_to_email(&self, user_id: i32, email: &str, verification_code: &str) -> anyhow::Result<UserAccountBinding> {
        // Validate email verification code
        {
            let codes = self.verification_codes.read().await;
            if let Some(stored_code) = codes.get(email) {
                if stored_code.expires_at < Utc::now() {
                    return Err(anyhow::anyhow!("验证码已过期"));
                }
                if stored_code.code != verification_code {
                    return Err(anyhow::anyhow!("验证码错误"));
                }
            } else {
                return Err(anyhow::anyhow!("验证码不存在或已过期"));
            }
        }

        // Get wallet user information
        let wallet_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.db_pool)
            .await?;

        // Check if this email is already bound to another user
        let existing_email_user = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT id FROM users WHERE email = $1 AND id != $2"
        )
        .bind(email)
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?;

        if existing_email_user.is_some() {
            return Err(anyhow::anyhow!("此邮箱地址已绑定到其他账户 / This email address is already bound to another account"));
        }

        // Check if user already has an email binding
        let existing_binding = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT id FROM user_account_bindings WHERE primary_user_id = $1 AND binding_type = 'wallet_to_email' AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?;

        if existing_binding.is_some() {
            return Err(anyhow::anyhow!("用户已有邮箱绑定 / User already has an email binding"));
        }

        // Update user's email
        sqlx::query("UPDATE users SET email = $1, updated_at = NOW() WHERE id = $2")
            .bind(email)
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;

        // Create binding record
        let binding = sqlx::query_as::<_, UserAccountBinding>(
            r#"
            INSERT INTO user_account_bindings (primary_user_id, binding_type, email, is_active)
            VALUES ($1, 'wallet_to_email', $2, true)
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(email)
        .fetch_one(&self.db_pool)
        .await?;

        // Remove used verification code
        {
            let mut codes = self.verification_codes.write().await;
            codes.remove(email);
        }

        tracing::info!("Successfully bound email {} to user {}", email, user_id);
        Ok(binding)
    }

    pub async fn get_user_bindings(&self, user_id: i32) -> anyhow::Result<UserBindingsResponse> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.db_pool)
            .await?;

        let bindings = sqlx::query_as::<_, UserAccountBinding>(
            "SELECT * FROM user_account_bindings WHERE primary_user_id = $1 AND is_active = true ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await?;

        let has_email_binding = !user.email.is_empty();
        let has_wallet_binding = user.ethereum_address.is_some() && !user.ethereum_address.as_ref().unwrap().is_empty();

        Ok(UserBindingsResponse {
            user_id,
            email: if has_email_binding { Some(user.email) } else { None },
            ethereum_address: user.ethereum_address,
            has_email_binding,
            has_wallet_binding,
            bindings,
        })
    }

    pub async fn unbind_account(&self, user_id: i32, binding_type: &str) -> anyhow::Result<()> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.db_pool)
            .await?;

        match binding_type {
            "email" => {
                // Check if user has email binding
                let has_email = !user.email.is_empty();
                if !has_email {
                    return Err(anyhow::anyhow!("用户没有邮箱绑定可以移除 / User has no email binding to remove"));
                }

                // Check if this is the only authentication method
                if user.ethereum_address.is_none() || user.ethereum_address.as_ref().unwrap().is_empty() {
                    return Err(anyhow::anyhow!("无法移除邮箱绑定：这是唯一的认证方式 / Cannot remove email binding: it's the only authentication method"));
                }

                // Remove email from user and deactivate binding
                sqlx::query("UPDATE users SET email = '', updated_at = NOW() WHERE id = $1")
                    .bind(user_id)
                    .execute(&self.db_pool)
                    .await?;

                sqlx::query("UPDATE user_account_bindings SET is_active = false WHERE primary_user_id = $1 AND binding_type IN ('email_to_wallet', 'wallet_to_email') AND email IS NOT NULL")
                    .bind(user_id)
                    .execute(&self.db_pool)
                    .await?;

                tracing::info!("Removed email binding for user {}", user_id);
            }
            "wallet" => {
                // Check if user has wallet binding
                let has_wallet = user.ethereum_address.is_some() && !user.ethereum_address.as_ref().unwrap().is_empty();
                if !has_wallet {
                    return Err(anyhow::anyhow!("用户没有钱包绑定可以移除 / User has no wallet binding to remove"));
                }

                // Check if this is the only authentication method
                if user.email.is_empty() {
                    return Err(anyhow::anyhow!("无法移除钱包绑定：这是唯一的认证方式 / Cannot remove wallet binding: it's the only authentication method"));
                }

                // Remove ethereum_address from user and deactivate binding
                sqlx::query("UPDATE users SET ethereum_address = NULL, updated_at = NOW() WHERE id = $1")
                    .bind(user_id)
                    .execute(&self.db_pool)
                    .await?;

                sqlx::query("UPDATE user_account_bindings SET is_active = false WHERE primary_user_id = $1 AND binding_type IN ('email_to_wallet', 'wallet_to_email') AND ethereum_address IS NOT NULL")
                    .bind(user_id)
                    .execute(&self.db_pool)
                    .await?;

                tracing::info!("Removed wallet binding for user {}", user_id);
            }
            _ => return Err(anyhow::anyhow!("无效的绑定类型：必须是 'email' 或 'wallet' / Invalid binding type: must be 'email' or 'wallet'")),
        }

        Ok(())
    }

    pub async fn find_user_by_binding(&self, email: Option<&str>, ethereum_address: Option<&str>) -> anyhow::Result<Option<User>> {
        if let Some(email) = email {
            // First try to find user by email directly
            if let Ok(user) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1 AND is_active = true")
                .bind(email)
                .fetch_one(&self.db_pool)
                .await
            {
                return Ok(Some(user));
            }

            // Then check bindings table
            if let Ok(binding) = sqlx::query_as::<_, UserAccountBinding>(
                "SELECT * FROM user_account_bindings WHERE email = $1 AND is_active = true"
            )
            .bind(email)
            .fetch_one(&self.db_pool)
            .await
            {
                if let Ok(user) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
                    .bind(binding.primary_user_id)
                    .fetch_one(&self.db_pool)
                    .await
                {
                    return Ok(Some(user));
                }
            }
        }

        if let Some(address) = ethereum_address {
            let address = address.to_lowercase();
            
            // First try to find user by ethereum_address directly
            if let Ok(user) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE ethereum_address = $1 AND is_active = true")
                .bind(&address)
                .fetch_one(&self.db_pool)
                .await
            {
                return Ok(Some(user));
            }

            // Then check bindings table
            if let Ok(binding) = sqlx::query_as::<_, UserAccountBinding>(
                "SELECT * FROM user_account_bindings WHERE ethereum_address = $1 AND is_active = true"
            )
            .bind(&address)
            .fetch_one(&self.db_pool)
            .await
            {
                if let Ok(user) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
                    .bind(binding.primary_user_id)
                    .fetch_one(&self.db_pool)
                    .await
                {
                    return Ok(Some(user));
                }
            }
        }

        Ok(None)
    }
}
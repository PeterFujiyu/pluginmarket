use crate::utils::config::SmtpConfig;
use anyhow::Result;

pub struct SmtpService {
    config: SmtpConfig,
}

impl SmtpService {
    pub fn new(config: SmtpConfig) -> Self {
        Self { config }
    }

    pub async fn send_verification_code(&self, email: &str, code: &str) -> Result<bool> {
        if !self.config.enabled {
            // SMTP not configured, return false to indicate code should be displayed
            tracing::info!("SMTP not enabled, verification code will be displayed: {}", code);
            return Ok(false);
        }

        // Validate SMTP configuration
        if self.config.username.is_empty() || self.config.password.is_empty() {
            tracing::warn!("SMTP credentials not configured, falling back to display mode");
            return Ok(false);
        }

        let subject = "GeekTools 插件市场 - 邮箱验证码";
        let body = self.create_verification_email_body(code);

        match self.send_email(email, subject, &body).await {
            Ok(_) => {
                tracing::info!("Verification code sent successfully to {}", email);
                Ok(true)
            }
            Err(e) => {
                tracing::error!("Failed to send verification code to {}: {}", email, e);
                // Fallback to display mode on error
                Ok(false)
            }
        }
    }

    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        use lettre::message::header::ContentType;
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{Message, SmtpTransport, Transport};

        // Build email message
        let email = Message::builder()
            .from(format!("{} <{}>", self.config.from_name, self.config.from_address).parse()?)
            .to(to.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body.to_string())?;

        // Configure SMTP transport
        let creds = Credentials::new(self.config.username.clone(), self.config.password.clone());
        
        let transport = if self.config.use_tls {
            // For Gmail: port 465 uses direct SSL, port 587 uses STARTTLS
            if self.config.port == 465 {
                SmtpTransport::relay(&self.config.host)?
                    .port(self.config.port)
                    .credentials(creds)
                    .timeout(Some(std::time::Duration::from_secs(30)))
                    .tls(lettre::transport::smtp::client::Tls::Wrapper(
                        lettre::transport::smtp::client::TlsParameters::new(self.config.host.clone())?
                    ))
                    .build()
            } else {
                // Use STARTTLS for other ports (like 587) - Gmail recommended
                SmtpTransport::starttls_relay(&self.config.host)?
                    .port(self.config.port)
                    .credentials(creds)
                    .timeout(Some(std::time::Duration::from_secs(30)))
                    .build()
            }
        } else {
            SmtpTransport::relay(&self.config.host)?
                .port(self.config.port)
                .credentials(creds)
                .timeout(Some(std::time::Duration::from_secs(30)))
                .build()
        };

        // Send email
        transport.send(&email)?;

        Ok(())
    }

    fn create_verification_email_body(&self, code: &str) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>邮箱验证码</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 600px;
            margin: 0 auto;
            background-color: white;
            padding: 40px;
            border-radius: 12px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .logo {{
            font-size: 24px;
            font-weight: bold;
            color: #FF8C47;
            margin-bottom: 10px;
        }}
        .title {{
            font-size: 20px;
            color: #2F2F2F;
            margin-bottom: 20px;
        }}
        .code-container {{
            background-color: #f8f9fa;
            border: 2px dashed #FF8C47;
            border-radius: 8px;
            padding: 20px;
            text-align: center;
            margin: 30px 0;
        }}
        .verification-code {{
            font-size: 32px;
            font-weight: bold;
            color: #FF8C47;
            letter-spacing: 4px;
            font-family: 'Courier New', monospace;
        }}
        .description {{
            color: #666;
            margin-bottom: 20px;
        }}
        .warning {{
            background-color: #fff3cd;
            border: 1px solid #ffeaa7;
            border-radius: 6px;
            padding: 15px;
            margin: 20px 0;
            color: #856404;
        }}
        .footer {{
            text-align: center;
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #eee;
            color: #999;
            font-size: 14px;
        }}
        .button {{
            display: inline-block;
            background-color: #FF8C47;
            color: white;
            padding: 12px 24px;
            text-decoration: none;
            border-radius: 6px;
            margin: 20px 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo">🔧 GeekTools</div>
            <div class="title">插件市场邮箱验证</div>
        </div>
        
        <div class="description">
            您好！欢迎使用 GeekTools 插件市场。为了验证您的邮箱地址，请使用以下验证码：
        </div>
        
        <div class="code-container">
            <div class="verification-code">{}</div>
        </div>
        
        <div class="description">
            请在 10 分钟内使用此验证码完成邮箱验证。如果您没有请求此验证码，请忽略此邮件。
        </div>
        
        <div class="warning">
            <strong>安全提示：</strong>
            <ul>
                <li>验证码仅用于本次登录验证，请勿分享给他人</li>
                <li>GeekTools 团队不会通过邮件、电话等方式主动询问您的验证码</li>
                <li>如有疑问，请联系我们的技术支持</li>
            </ul>
        </div>
        
        <div class="footer">
            <p>此邮件由 GeekTools 插件市场自动发送，请勿回复</p>
            <p>© 2025 GeekTools Plugin Marketplace. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
            "#,
            code
        )
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled && !self.config.username.is_empty() && !self.config.password.is_empty()
    }
}

// Test helper functions
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> SmtpConfig {
        SmtpConfig {
            enabled: true,
            host: "smtp.gmail.com".to_string(),
            port: 587,
            username: "test@example.com".to_string(),
            password: "test_password".to_string(),
            from_address: "noreply@geektools.dev".to_string(),
            from_name: "GeekTools Test".to_string(),
            use_tls: true,
        }
    }

    #[test]
    fn test_smtp_service_creation() {
        let config = create_test_config();
        let service = SmtpService::new(config);
        assert!(service.is_enabled());
    }

    #[test]
    fn test_disabled_smtp() {
        let mut config = create_test_config();
        config.enabled = false;
        let service = SmtpService::new(config);
        assert!(!service.is_enabled());
    }

    #[test]
    fn test_verification_email_body() {
        let config = create_test_config();
        let service = SmtpService::new(config);
        let body = service.create_verification_email_body("123456");
        
        assert!(body.contains("123456"));
        assert!(body.contains("GeekTools"));
        assert!(body.contains("验证码"));
    }
}
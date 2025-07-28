# SMTP Configuration Troubleshooting Guide

This guide helps resolve common SMTP configuration issues in the GeekTools Plugin Marketplace.

## Common Error Messages and Solutions

### 1. Certificate Not Valid for Name

**Error**: `certificate not valid for name "smtp.gamil.com"`

**Cause**: Typo in SMTP hostname or wrong certificate

**Solutions**:
- **Fix typos**: Common mistakes:
  - `smtp.gamil.com` → `smtp.gmail.com`
  - `gmail.com` → `smtp.gmail.com`
  - `outlook.com` → `smtp-mail.outlook.com`
- **Use correct hostnames**:
  - Gmail: `smtp.gmail.com`
  - Outlook: `smtp-mail.outlook.com`
  - Yahoo: `smtp.mail.yahoo.com`

### 2. Authentication Failed

**Error**: `Authentication failed`

**Solutions**:
- **Gmail**: Use App Passwords (not your regular password)
  1. Enable 2-Factor Authentication
  2. Go to Google Account > Security > App passwords
  3. Generate a new app password
  4. Use this password in SMTP configuration

- **Outlook**: Use your Microsoft account credentials
  - Username: Your full email address
  - Password: Your account password

### 3. Connection Timeout

**Error**: `Connection timeout` or `Cannot connect to SMTP server`

**Solutions**:
- Check firewall settings
- Verify SMTP hostname and port
- Common ports:
  - Port 587: STARTTLS (recommended)
  - Port 465: SSL/TLS
  - Port 25: Plain (usually blocked)

## Provider-Specific Configuration

### Gmail Configuration

```env
SMTP_ENABLED=true
SMTP_HOST=smtp.gmail.com  
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-16-char-app-password
SMTP_FROM_ADDRESS=your-email@gmail.com
SMTP_FROM_NAME=Your App Name
```

**Requirements**:
- 2-Factor Authentication enabled
- App Password generated (not regular password)
- Less secure app access is NOT needed with app passwords

### Outlook/Hotmail Configuration

```env
SMTP_ENABLED=true
SMTP_HOST=smtp-mail.outlook.com
SMTP_PORT=587
SMTP_USERNAME=your-email@outlook.com
SMTP_PASSWORD=your-account-password
SMTP_FROM_ADDRESS=your-email@outlook.com
SMTP_FROM_NAME=Your App Name
```

### Yahoo Configuration

```env
SMTP_ENABLED=true
SMTP_HOST=smtp.mail.yahoo.com
SMTP_PORT=587
SMTP_USERNAME=your-email@yahoo.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_ADDRESS=your-email@yahoo.com
SMTP_FROM_NAME=Your App Name
```

**Requirements**:
- App Password required (similar to Gmail)

### Custom SMTP Server

```env
SMTP_ENABLED=true
SMTP_HOST=mail.yourdomain.com
SMTP_PORT=587
SMTP_USERNAME=your-username
SMTP_PASSWORD=your-password
SMTP_FROM_ADDRESS=noreply@yourdomain.com
SMTP_FROM_NAME=Your App Name
```

## Testing SMTP Configuration

### Manual Testing with Docker

1. **Access the application container**:
   ```bash
   docker-compose exec app bash
   ```

2. **Test SMTP connectivity**:
   ```bash
   # Test basic connectivity
   nc -zv smtp.gmail.com 587
   
   # Test with telnet (if available)
   telnet smtp.gmail.com 587
   ```

### Using External Tools

1. **Online SMTP Test Tools**:
   - MXToolbox SMTP Test
   - Mail-Tester.com

2. **Command Line Tools**:
   ```bash
   # Test with curl
   curl -v --url 'smtps://smtp.gmail.com:587' \
        --mail-from 'sender@gmail.com' \
        --mail-rcpt 'recipient@example.com' \
        --user 'sender@gmail.com:app-password'
   ```

## Configuration Management

### Updating SMTP Configuration

1. **Edit the configuration file**:
   ```bash
   nano data/config.env
   ```

2. **Restart the application**:
   ```bash
   docker-compose restart app
   ```

### Disabling SMTP

If you want to disable SMTP temporarily:

```env
SMTP_ENABLED=false
```

The application will display verification codes in the logs instead of sending emails.

## Troubleshooting Steps

### Step 1: Verify Configuration
```bash
# Check your configuration
cat data/config.env | grep SMTP
```

### Step 2: Check Application Logs
```bash
# View application logs
docker-compose logs app | grep -i smtp
```

### Step 3: Test Network Connectivity
```bash
# Test from host machine
nc -zv smtp.gmail.com 587

# Test from container
docker-compose exec app nc -zv smtp.gmail.com 587
```

### Step 4: Verify Credentials
- Ensure username/password are correct
- For Gmail: Use app password, not regular password
- Check if 2FA is enabled where required

### Step 5: Check Provider Requirements
- Some providers require app-specific passwords
- Some providers block connections from certain IP ranges
- Check if your server IP is blacklisted

## Security Considerations

### Best Practices
- Always use TLS/STARTTLS (port 587)
- Use app-specific passwords when available
- Don't use your main account password
- Regularly rotate SMTP passwords
- Monitor failed login attempts

### Rate Limiting
Most providers have sending limits:
- Gmail: 500 emails/day (free), 2000/day (paid)
- Outlook: 300 emails/day
- Yahoo: 500 emails/day

## Getting Help

If you continue to experience issues:

1. **Check the application logs**: `docker-compose logs app`
2. **Verify network connectivity**: Use `nc` or `telnet`
3. **Test with a different email provider**: Try Gmail with app password
4. **Contact your email provider**: They may have specific requirements

## Common Solutions Summary

| Issue | Solution |
|-------|----------|
| Typo in hostname | Fix to correct SMTP server name |
| Authentication failed | Use app password for Gmail/Yahoo |
| Connection timeout | Check firewall, use port 587 |
| Certificate error | Verify correct hostname |
| Rate limiting | Reduce email frequency |
| IP blocked | Contact provider or use different server |

Remember: The install script now includes validation and typo correction to prevent most of these issues during initial setup.
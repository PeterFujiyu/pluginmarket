#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
测试SMTP配置的Python脚本
"""

import smtplib
import ssl
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart

def test_smtp_connection():
    # 从config.yaml中读取的SMTP配置
    smtp_host = "smtp.qq.com"
    smtp_port = 587
    username = "jiyu.fu@qq.com"
    password = "hwtnihpollwgbhac"  # 你的App Password
    from_address = "jiyu.fu@qq.com"
    from_name = "GeekTools Plugin Marketplace"
    to_email = "jiyu.fu.369@outlook.com"  # 测试接收邮箱
    
    print(f"正在测试SMTP连接...")
    print(f"服务器: {smtp_host}:{smtp_port}")
    print(f"用户名: {username}")
    print(f"发送到: {to_email}")
    print("-" * 50)
    
    try:
        # 创建邮件内容
        message = MIMEMultipart()
        message["From"] = f"{from_name} <{from_address}>"
        message["To"] = to_email
        message["Subject"] = "SMTP测试邮件"
        
        # 邮件正文
        body = """
        这是一封SMTP配置测试邮件。
        
        如果你收到了这封邮件，说明SMTP配置是正确的。
        
        测试验证码：123456
        
        ——来自GeekTools插件市场
        """
        
        message.attach(MIMEText(body, "plain", "utf-8"))
        
        # 连接到SMTP服务器
        print("1. 连接到SMTP服务器...")
        server = smtplib.SMTP(smtp_host, smtp_port)
        
        # 启用调试模式，显示详细信息
        server.set_debuglevel(1)
        
        print("2. 启动TLS加密...")
        server.starttls()
        
        print("3. 登录SMTP服务器...")
        server.login(username, password)
        
        print("4. 发送邮件...")
        text = message.as_string()
        server.sendmail(from_address, to_email, text)
        
        print("5. 关闭连接...")
        server.quit()
        
        print("✅ 邮件发送成功！")
        print("请检查接收邮箱是否收到测试邮件。")
        
    except smtplib.SMTPAuthenticationError as e:
        print(f"❌ 认证失败: {e}")
        print("可能的原因:")
        print("- App Password不正确")
        print("- Gmail账户未启用2FA")
        print("- App Password已过期")
        
    except smtplib.SMTPConnectError as e:
        print(f"❌ 连接失败: {e}")
        print("可能的原因:")
        print("- 网络连接问题")
        print("- SMTP服务器地址或端口错误")
        
    except smtplib.SMTPException as e:
        print(f"❌ SMTP错误: {e}")
        
    except Exception as e:
        print(f"❌ 其他错误: {e}")

def test_connection_only():
    """只测试SMTP连接，不发送邮件"""
    smtp_host = "smtp.gmail.com"
    smtp_port = 587
    username = "peter.fu.369@gmail.com"
    password = "wkhr eucs gold vyhc"
    
    print("测试SMTP连接（不发送邮件）...")
    
    try:
        server = smtplib.SMTP(smtp_host, smtp_port)
        server.set_debuglevel(1)
        server.starttls()
        server.login(username, password)
        server.quit()
        print("✅ SMTP连接和认证成功！")
        return True
    except Exception as e:
        print(f"❌ 连接测试失败: {e}")
        return False

if __name__ == "__main__":
    print("=== GeekTools SMTP配置测试 ===")
    print()
    
    # 先测试连接
    if test_connection_only():
        print()
        print("连接成功，现在尝试发送测试邮件...")
        print()
        test_smtp_connection()
    else:
        print()
        print("连接失败，请检查SMTP配置。")
use anyhow::{Context, Result};
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use rustls::{Certificate as RustlsCert, PrivateKey};

/// 证书配置
pub struct CertConfig {
    pub cert: RustlsCert,
    pub key: PrivateKey,
}

impl CertConfig {
    /// 生成自签名证书用于测试
    pub fn generate_self_signed() -> Result<Self> {
        let mut params = CertificateParams::new(vec!["localhost".to_string()]);
        params.distinguished_name = DistinguishedName::new();
        params.distinguished_name.push(
            rcgen::DnType::CommonName,
            rcgen::DnValue::Utf8String("localhost".to_string()),
        );

        let cert = Certificate::from_params(params)
            .context("Failed to generate certificate")?;

        let cert_der = cert.serialize_der()
            .context("Failed to serialize certificate")?;
        let key_der = cert.serialize_private_key_der();

        Ok(CertConfig {
            cert: RustlsCert(cert_der),
            key: PrivateKey(key_der),
        })
    }
}

/// 创建客户端TLS配置
pub fn create_client_config() -> Result<rustls::ClientConfig> {
    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(rustls::RootCertStore::empty())
        .with_no_client_auth();
    
    Ok(config)
}

/// 创建服务端TLS配置
pub fn create_server_config(cert_config: CertConfig) -> Result<rustls::ServerConfig> {
    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert_config.cert], cert_config.key)
        .context("Failed to build server config")?;

    Ok(config)
}
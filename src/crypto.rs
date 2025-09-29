use anyhow::{Context, Result};
use quinn::{ClientConfig, ServerConfig, TransportConfig};
use rustls::{Certificate, ClientConfig as RustlsClientConfig, PrivateKey, ServerConfig as RustlsServerConfig};
use std::{fs, path::Path, sync::Arc, time::Duration};

pub struct CertConfig {
    pub cert: Certificate,
    pub key: PrivateKey,
    pub cert_pem: String,
}

impl CertConfig {
    pub fn generate_self_signed() -> Result<Self> {
        use rcgen::{Certificate as RcgenCert, CertificateParams, DistinguishedName};
        
        let mut params = CertificateParams::new(vec!["localhost".to_string()]);
        params.distinguished_name = DistinguishedName::new();
        params.distinguished_name.push(rcgen::DnType::CommonName, "T3XT Server");
        
        let cert = RcgenCert::from_params(params)
            .context("Failed to generate certificate")?;
        
        let cert_der = cert.serialize_der()
            .context("Failed to serialize certificate")?;
        let key_der = cert.serialize_private_key_der();
        
        // 生成PEM格式
        let cert_pem = cert.serialize_pem()
            .context("Failed to serialize certificate to PEM")?;
        
        // 创建certs目录
        fs::create_dir_all("certs").context("Failed to create certs directory")?;
        
        // 保存证书文件
        fs::write("certs/server.crt", &cert_pem)
            .context("Failed to write certificate file")?;
        
        // 保存私钥文件
        let key_pem = cert.serialize_private_key_pem();
        fs::write("certs/server.key", &key_pem)
            .context("Failed to write private key file")?;
        
        println!("🔐 证书已保存到:");
        println!("   📄 证书文件: certs/server.crt");
        println!("   🔑 私钥文件: certs/server.key");
        
        Ok(Self {
            cert: Certificate(cert_der),
            key: PrivateKey(key_der),
            cert_pem,
        })
    }
    
    pub fn load_from_files() -> Result<Self> {
        let cert_pem = fs::read_to_string("certs/server.crt")
            .context("Failed to read certificate file")?;
        let key_pem = fs::read_to_string("certs/server.key")
            .context("Failed to read private key file")?;
        
        // 解析证书
        let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
            .context("Failed to parse certificate")?
            .into_iter()
            .next()
            .context("No certificate found")?;
        
        // 解析私钥
        let key_der = rustls_pemfile::pkcs8_private_keys(&mut key_pem.as_bytes())
            .context("Failed to parse private key")?
            .into_iter()
            .next()
            .context("No private key found")?;
        
        Ok(Self {
            cert: Certificate(cert_der),
            key: PrivateKey(key_der),
            cert_pem,
        })
    }
    
    pub fn get_or_create() -> Result<Self> {
        if Path::new("certs/server.crt").exists() && Path::new("certs/server.key").exists() {
            println!("📄 使用现有证书文件");
            Self::load_from_files()
        } else {
            println!("🔧 生成新的自签名证书");
            Self::generate_self_signed()
        }
    }
}

pub fn create_server_config(cert_config: CertConfig) -> Result<RustlsServerConfig> {
    let config = RustlsServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert_config.cert], cert_config.key)
        .context("Failed to create server config")?;
    
    Ok(config)
}

pub fn create_insecure_client_config() -> Result<RustlsClientConfig> {
    let config = RustlsClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
        .with_no_client_auth();
    
    Ok(config)
}

pub fn create_client_config_with_cert(cert_path: &str) -> Result<RustlsClientConfig> {
    let cert_pem = fs::read_to_string(cert_path)
        .context("Failed to read certificate file")?;
    
    let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
        .context("Failed to parse certificate")?
        .into_iter()
        .next()
        .context("No certificate found")?;
    
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add(&Certificate(cert_der))
        .context("Failed to add certificate to root store")?;
    
    let config = RustlsClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    
    Ok(config)
}

pub fn create_quinn_client_config(rustls_config: RustlsClientConfig) -> ClientConfig {
    let mut transport = TransportConfig::default();
    transport.max_idle_timeout(Some(Duration::from_secs(30).try_into().unwrap()));
    transport.keep_alive_interval(Some(Duration::from_secs(5)));
    
    let mut config = ClientConfig::new(Arc::new(rustls_config));
    config.transport_config(Arc::new(transport));
    config
}

struct SkipServerVerification;

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}
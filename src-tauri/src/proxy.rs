use crate::tooling::ProxyRule;
use hyper::client::HttpConnector;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, Uri};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::server::{ClientHello, ResolvesServerCert};
use tokio_rustls::rustls::sign::CertifiedKey;

#[derive(Default, Debug)]
struct ProxyState {
    rules: Arc<RwLock<Vec<ProxyRule>>>,
    running: bool,
}

static STATE: once_cell::sync::Lazy<Mutex<ProxyState>> =
    once_cell::sync::Lazy::new(|| Mutex::new(ProxyState::default()));

pub fn apply_rules(rules: Vec<ProxyRule>) -> Result<(), String> {
    {
        let mut state = STATE.lock().expect("proxy state lock");
        *state.rules.write().expect("rules lock") = rules;
        if !state.running {
            state.running = true;
            spawn_proxy(state.rules.clone());
        }
    }
    Ok(())
}

fn spawn_proxy(rules: Arc<RwLock<Vec<ProxyRule>>>) {
    thread::spawn(move || {
        let runtime = Runtime::new().expect("proxy runtime");
        runtime.block_on(async move {
            let http_rules = rules.clone();
            let http_client: Client<HttpConnector, Body> = Client::new();
            
            // HTTP Proxy
            tokio::spawn(async move {
                let addr: SocketAddr = "127.0.0.1:8080".parse().expect("addr");
                let make_svc = make_service_fn(move |_| {
                    let client = http_client.clone();
                    let rules = http_rules.clone();
                    async move {
                        Ok::<_, Infallible>(service_fn(move |req| {
                            proxy_request(req, client.clone(), rules.clone())
                        }))
                    }
                });
                let server = Server::bind(&addr).serve(make_svc);
                if let Err(err) = server.await {
                    eprintln!("proxy server error: {err}");
                }
            });

            // HTTPS/TLS Proxy
            if let Ok(tls_config) = load_tls_config() {
                let acceptor = TlsAcceptor::from(Arc::new(tls_config));
                let listener = match TcpListener::bind("127.0.0.1:8443").await {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!("proxy tls bind error: {}", e);
                        return;
                    }
                };
                
                loop {
                    let (stream, _) = match listener.accept().await {
                        Ok(value) => value,
                        Err(err) => {
                            eprintln!("proxy tls accept error: {err}");
                            continue;
                        }
                    };
                    let acceptor = acceptor.clone();
                    let client = Client::new();
                    let rules = rules.clone();
                    tokio::spawn(async move {
                        match acceptor.accept(stream).await {
                            Ok(tls_stream) => {
                                let service = service_fn(move |req| {
                                    proxy_request(req, client.clone(), rules.clone())
                                });
                                if let Err(err) =
                                    hyper::server::conn::Http::new()
                                        .serve_connection(tls_stream, service)
                                        .await
                                {
                                    eprintln!("proxy tls error: {err}");
                                }
                            }
                            Err(err) => {
                                eprintln!("proxy tls handshake error: {err}");
                            }
                        }
                    });
                }
            } else {
                eprintln!("proxy tls config failed to load");
            }
        });
    });
}

async fn proxy_request(
    req: Request<Body>,
    client: Client<HttpConnector, Body>,
    rules: Arc<RwLock<Vec<ProxyRule>>>,
) -> Result<Response<Body>, Infallible> {
    let host = req
        .headers()
        .get("host")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let path = req.uri().path().to_string();
    
    let target = {
        let rules = rules.read().expect("rules lock");
        rules
            .iter()
            .find(|rule| host_matches(&host, rule) && path.starts_with(&rule.path))
            .map(|rule| rule.target.clone())
    };
    
    let target = match target {
        Some(value) => value,
        None => {
            return Ok(Response::builder()
                .status(404)
                .body(Body::from("proxy rule not found"))
                .unwrap())
        }
    };

    match build_target_uri(&target, req.uri()) {
        Ok(uri) => {
            let (mut parts, body) = req.into_parts();
            parts.uri = uri;
            if let Some(authority) = parts.uri.authority() {
                if let Ok(val) = authority.as_str().parse() {
                     parts.headers.insert("host", val);
                }
            }
            match client.request(Request::from_parts(parts, body)).await {
                Ok(response) => Ok(response),
                Err(err) => Ok(Response::builder()
                    .status(502)
                    .body(Body::from(format!("proxy error: {err}")))
                    .unwrap()),
            }
        }
        Err(err) => Ok(Response::builder()
            .status(500)
            .body(Body::from(format!("proxy error: {err}")))
            .unwrap()),
    }
}

fn host_matches(host: &str, rule: &ProxyRule) -> bool {
    let host = host.split(':').next().unwrap_or(host);
    host == rule.host
}

fn build_target_uri(base: &str, original: &Uri) -> Result<Uri, String> {
    let base_uri: Uri = base.parse().map_err(|e: hyper::http::uri::InvalidUri| e.to_string())?;
    let scheme = base_uri.scheme_str().ok_or_else(|| "missing scheme".to_string())?;
    let authority = base_uri
        .authority()
        .ok_or_else(|| "missing authority".to_string())?
        .as_str();
    let path_and_query = original
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/");
    let uri_str = format!("{scheme}://{authority}{path_and_query}");
    uri_str.parse::<Uri>().map_err(|e| e.to_string())
}

fn load_tls_config() -> Result<rustls::ServerConfig, String> {
    // Explicitly set the provider (using ring implicitly via features or explicit call)
    // In rustls 0.23 with ring feature, we should check if default provider works.
    // If not, we might need: rustls::crypto::ring::default_provider().install_default().ok();
    // But typically Builder::new() works if safe defaults are available.
    
    let cert_path = PathBuf::from("app/certs/proxy.crt");
    let key_path = PathBuf::from("app/certs/proxy.key");
    let default = load_certified_key(&cert_path, &key_path)?;
    let resolver = Arc::new(SniResolver::new(default, PathBuf::from("app/certs")));
    
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(resolver);
    Ok(config)
}

fn load_certified_key(cert_path: &PathBuf, key_path: &PathBuf) -> Result<CertifiedKey, String> {
    if !cert_path.exists() || !key_path.exists() {
        return Err("proxy cert/key not found".to_string());
    }
    let cert_bytes = std::fs::read(cert_path).map_err(|e| e.to_string())?;
    let key_bytes = std::fs::read(key_path).map_err(|e| e.to_string())?;
    let mut cert_reader = std::io::Cursor::new(cert_bytes);
    let mut key_reader = std::io::Cursor::new(key_bytes);
    
    let certs = rustls_pemfile::certs(&mut cert_reader)
        .map(|res| res.map_err(|_| "invalid cert".to_string()))
        .collect::<Result<Vec<CertificateDer<'static>>, String>>()?;
        
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
        .map(|res| res.map(|key| PrivateKeyDer::Pkcs8(key)).map_err(|_| "invalid key".to_string()))
        .collect::<Result<Vec<PrivateKeyDer<'static>>, String>>()?;
        
    let key = keys.pop().ok_or_else(|| "proxy key missing".to_string())?;
    
    // rustls 0.23: use crypto provider to create signing key
    let signing_key = rustls::crypto::ring::sign::any_supported_type(&key)
        .map_err(|_| "unsupported proxy key".to_string())?;
        
    Ok(CertifiedKey::new(certs, signing_key))
}

#[derive(Debug)]
struct SniResolver {
    default: Arc<CertifiedKey>,
    cache: Mutex<HashMap<String, Arc<CertifiedKey>>>,
    cert_dir: PathBuf,
}

impl SniResolver {
    fn new(default: CertifiedKey, cert_dir: PathBuf) -> Self {
        Self {
            default: Arc::new(default),
            cache: Mutex::new(HashMap::new()),
            cert_dir,
        }
    }

    fn load_for_host(&self, host: &str) -> Option<Arc<CertifiedKey>> {
        let mut cache = self.cache.lock().expect("sni cache lock");
        if let Some(entry) = cache.get(host) {
            return Some(entry.clone());
        }
        let variants = [host.to_string(), host.replace('.', "-")];
        for variant in variants {
            let cert_path = self.cert_dir.join(format!("{variant}.crt"));
            let key_path = self.cert_dir.join(format!("{variant}.key"));
            if let Ok(key) = load_certified_key(&cert_path, &key_path) {
                let entry = Arc::new(key);
                cache.insert(host.to_string(), entry.clone());
                return Some(entry);
            }
        }
        None
    }
}

impl ResolvesServerCert for SniResolver {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<CertifiedKey>> {
        if let Some(server_name) = client_hello.server_name() {
            if let Some(entry) = self.load_for_host(server_name) {
                return Some(entry);
            }
        }
        Some(self.default.clone())
    }
}


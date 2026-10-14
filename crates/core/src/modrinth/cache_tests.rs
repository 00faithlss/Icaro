// SPDX-License-Identifier: GPL-3.0-only
use super::*;
use serde_json::{json, Value};
use std::collections::VecDeque;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};

struct Api {
    catalog: Catalog,
    requests: Arc<Mutex<Vec<(String, Value)>>>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Api { fn drop(&mut self) { self.task.abort(); } }
impl Api {
    async fn new(replies: Vec<(u16, &'static str, Value)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(vec![]));
        let received = requests.clone();
        let mut replies: VecDeque<_> = replies.into();
        let task = tokio::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = vec![];
                let (end, length) = loop {
                    let mut chunk = [0;4096];
                    let n = stream.read(&mut chunk).await.unwrap();
                    assert!(n > 0);
                    request.extend_from_slice(&chunk[..n]);
                    if let Some(end) = request.windows(4).position(|s| s == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&request[..end]);
                        let length = headers.lines().find_map(|line| line.split_once(':')
                            .filter(|(name,_)| name.eq_ignore_ascii_case("content-length"))
                            .map(|(_, value)| value.trim().parse::<usize>().unwrap())).unwrap_or(0);
                        break (end+4,length);
                    }
                };
                while request.len() < end+length {
                    let mut chunk = [0;4096];
                    let n = stream.read(&mut chunk).await.unwrap();
                    assert!(n > 0);
                    request.extend_from_slice(&chunk[..n]);
                }
                let path = String::from_utf8_lossy(&request[..end]).lines().next().unwrap().to_owned();
                let body = if length == 0 { Value::Null } else { serde_json::from_slice(&request[end..end+length]).unwrap() };
                received.lock().unwrap().push((path,body));
                let (status,headers,body) = replies.pop_front().unwrap_or((500,"",json!({"error":"unexpected request"})));
                let body = body.to_string();
                let response = format!("HTTP/1.1 {status} Test\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
                let _ = stream.write_all(response.as_bytes()).await;
            }
        });
        let mut catalog = Catalog::new().unwrap().with_metadata_cache();
        catalog.client = Client::builder().no_proxy().timeout(Duration::from_secs(2)).build().unwrap();
        catalog.api = format!("http://{addr}/v2/");
        Self { catalog, requests, task }
    }
    fn count(&self) -> usize { self.requests.lock().unwrap().len() }
    fn expire_cooldown(&self) { self.catalog.cooldown.lock().unwrap().until = Some(Instant::now()); }
}
fn project(id: &str) -> Value {
    json!({"id":id,"slug":id,"title":id,"description":"","body":"","license":{"id":"MIT","name":"MIT"},"downloads":0,"client_side":"optional"})
}
fn version(n: usize) -> Value {
    json!({"id":format!("v{n}"),"project_id":format!("p{n}"),"name":"Mod","version_number":"1","version_type":"release","game_versions":["1.21.1"],"loaders":["fabric"],"dependencies":[],"files":[{"filename":format!("p{n}.jar"),"url":"https://cdn.modrinth.com/data/mod.jar","size":123,"primary":true,"hashes":{"sha512":format!("{n:0128x}")}}]})
}
#[tokio::test]
async fn ciento_sesenta_y_tres_hashes_reintentos_429_y_cambio_consultan_solo_lo_pendiente() {
    let hashes: Vec<_> = (1..=163).map(|n| format!("{n:0128x}")).collect();
    let initial: serde_json::Map<_,_> = (1..163).map(|n| (format!("{n:0128x}"),version(n))).collect();
    let api = Api::new(vec![
        (200,"",Value::Object(initial)),
        (200,"",project("p1")),
        (429,"Retry-After: 120\r\n",json!({})),
        (429,"Retry-After: 1\r\n",json!({})),
        (200,"",project("p2")),
        (200,"",json!({format!("{:0128x}",164): version(164)})),
    ]).await;
    let catalog = &api.catalog;
    assert_eq!(catalog.versions_from_hashes(&hashes).await.unwrap().len(),162);
    catalog.project("p1").await.unwrap();
    assert_eq!(catalog.project("p2").await.unwrap_err(),Error::RateLimited(120));
    for _ in 0..3 {
        assert_eq!(catalog.clone().versions_from_hashes(&hashes).await.unwrap().len(),162);
        catalog.project("p1").await.unwrap();
        assert!(matches!(catalog.project("p2").await,Err(Error::RateLimited(_))));
    }
    assert_eq!(api.count(),3);
    api.expire_cooldown();
    assert_eq!(catalog.project("p2").await.unwrap_err(),Error::RateLimited(120));
    assert_eq!(api.count(),4);
    api.expire_cooldown();
    catalog.project("p2").await.unwrap();
    assert_eq!(catalog.cooldown.lock().unwrap().failures,0);
    let mut changed = hashes.clone();
    changed[0] = format!("{:0128x}",164);
    assert_eq!(catalog.versions_from_hashes(&changed).await.unwrap().len(),162);
    assert_eq!(api.requests.lock().unwrap()[5].1["hashes"],json!([format!("{:0128x}",164)]));
    assert_eq!(api.count(),6);
}
#[tokio::test]
async fn metadatos_compartidos_claves_por_filtros_y_fallos_no_cacheados() {
    let api = Api::new(vec![
        (503,"",json!({})),(200,"",project("p1")),
        (200,"",version(1)),(200,"",json!([version(1)])),(200,"",json!([])),
    ]).await;
    assert!(api.catalog.project("p1").await.is_err());
    let clone = api.catalog.clone();
    let (a,b) = tokio::join!(api.catalog.project("p1"),clone.project("p1"));
    assert_eq!(a.unwrap().id,b.unwrap().id);
    api.catalog.version("v1").await.unwrap();
    clone.version("v1").await.unwrap();
    assert_eq!(api.catalog.versions("p1",Some("1.21.1"),Some("fabric")).await.unwrap().len(),1);
    clone.versions("p1",Some("1.21.1"),Some("fabric")).await.unwrap();
    assert!(clone.versions("p1",Some("1.20.1"),Some("fabric")).await.unwrap().is_empty());
    assert_eq!(api.count(),5);
}
#[tokio::test]
async fn hash_no_confirmado_y_json_invalido_no_contaminan_la_cache() {
    let hash = format!("{:0128x}",1);
    let api = Api::new(vec![
        (200,"",json!({&hash:version(2)})),(200,"",json!({&hash:version(1)})),
        (200,"",json!({"malformed":true})),(200,"",project("p1")),
    ]).await;
    assert!(api.catalog.versions_from_hashes(&[hash.clone()]).await.is_err());
    assert_eq!(api.catalog.versions_from_hashes(&[hash]).await.unwrap().len(),1);
    assert!(api.catalog.project("p1").await.is_err());
    api.catalog.project("p1").await.unwrap();
    assert_eq!(api.count(),4);
}

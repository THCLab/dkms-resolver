use std::{
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use actix_web::{http::header, web, App, HttpResponse, HttpServer, Responder};
use kademlia_dht::{Key, Node, NodeData};
use keri::{
    database::sled::SledEventDatabase,
    event_message::signed_event_message::Message,
    event_parsing::message::signed_event_stream,
    prefix::{IdentifierPrefix, Prefix},
    processor::{event_storage::EventStorage, EventProcessor},
};
use rand::random;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha3::{Digest, Sha3_256};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {
    /// API listen port.
    #[structopt(long, env, default_value = "9599")]
    api_port: u16,

    /// API public host name. Announced in DHT together with API port.
    #[structopt(long, env, default_value = "localhost")]
    api_public_host: String,

    /// KERI database path.
    #[structopt(long, env, default_value = "db")]
    db_path: PathBuf,

    /// DHT listen port.
    #[structopt(long, env, default_value = "9145")]
    dht_port: u16,

    /// DHT bootstrap IP address.
    #[structopt(long, env)]
    dht_bootstrap_addr: Option<String>,
}

struct AppState {
    dht_node: Mutex<Node>,
    api_public_addr: SocketAddr,
    event_processor: Mutex<EventStorage>,
    witness_ips: Mutex<HashMap<String, WitnessIp>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WitnessIp {
    ip: SocketAddr,
}

#[actix_web::get("/key_states/{issuer_id}")]
async fn key_state_get(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let issuer_id = match IdentifierPrefix::from_str(&*path) {
        Ok(id) => id,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Invalid issuer ID: {:?}", err))
        }
    };
    let event_proc = data.event_processor.lock().unwrap();
    match event_proc.get_state(&issuer_id).unwrap() {
        Some(key_state) => return HttpResponse::Ok().json(key_state),
        None => {
            if let Some(addr) = data
                .dht_node
                .lock()
                .unwrap()
                .get(&get_dht_key(issuer_id.to_str().as_bytes()))
            {
                let url = format!("http://{}/key_states/{}", addr, issuer_id.to_str());
                log::info!("Found key state for {:?} at {:?}", issuer_id, url);
                if let Ok(resp) = reqwest::get(url).await {
                    let body = resp.text().await.unwrap();
                    let key_state: Value = serde_json::from_str(&body).unwrap();
                    return HttpResponse::Ok().json(key_state);
                }
            };
        }
    }
    return HttpResponse::NotFound().body(format!("Key state for {:?} not found", issuer_id));
}

#[actix_web::get("/key_logs/{issuer_id}")]
async fn key_log_get(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let issuer_id = match IdentifierPrefix::from_str(&*path) {
        Ok(id) => id,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Invalid issuer ID: {:?}", err))
        }
    };
    let event_proc = data.event_processor.lock().unwrap();
    match event_proc.get_kel(&issuer_id).unwrap() {
        Some(key_log) => {
            return HttpResponse::Ok()
                .insert_header(header::ContentType(mime::APPLICATION_OCTET_STREAM))
                .body(key_log)
        }
        None => {
            if let Some(addr) = data
                .dht_node
                .lock()
                .unwrap()
                .get(&get_dht_key(issuer_id.to_str().as_bytes()))
            {
                let url = format!("http://{}/key_logs/{}", addr, issuer_id.to_str());
                log::info!("Found key log for {:?} at {:?}", issuer_id, url);
                if let Ok(resp) = reqwest::get(url).await {
                    let body = resp.bytes().await.unwrap();
                    return HttpResponse::Ok()
                        .insert_header(header::ContentType(mime::APPLICATION_OCTET_STREAM))
                        .body(body);
                }
            };
        }
    }
    return HttpResponse::NotFound().body(format!("Key state for {:?} not found", issuer_id));
}

#[actix_web::post("/messages/{issuer_id}")]
async fn message_put(
    path: web::Path<String>,
    body: web::Bytes,
    data: web::Data<AppState>,
) -> impl Responder {
    let issuer_id = match IdentifierPrefix::from_str(&*path) {
        Ok(id) => id,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Invalid issuer ID: {:?}", err))
        }
    };
    let events = match signed_event_stream(&body) {
        Ok((_, events)) => events,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!(
                "Invalid key state event: {:?}",
                err.map(|err| err.1)
            ))
        }
    };
    
    let processor = EventProcessor::new(data.event_processor.lock().unwrap().db.clone());
    for event in events {
        let msg = match Message::try_from(event) {
            Ok(msg) => msg,
            Err(err) => {
                return HttpResponse::BadRequest()
                .body(format!("Invalid key state message: {:?}", err))
            }
        };
        match processor.process(msg) {
            Ok(success) => {
                log::info!(
                    "Saving event {data:?} for issuer {id:?}",
                    id = issuer_id,
                    data = success
                );
            }
            Err(err) => {
                return HttpResponse::BadRequest()
                .body(format!("Error while processing incoming event: {:?}", err))
            }
        };
    }
    
    {
        let mut node = data.dht_node.lock().unwrap();
        node.insert(
            get_dht_key(issuer_id.to_str().as_bytes()),
            &data.api_public_addr.to_string(),
        );
    };

    HttpResponse::Ok().json(json!({}))
}

#[actix_web::get("/witness_ips/{witness_id}")]
async fn witness_ip_get(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let witness_id = &*path;
    let witness_ips = data.witness_ips.lock().unwrap();
    match witness_ips.get(witness_id) {
        Some(witness_ip) => return HttpResponse::Ok().json(witness_ip),
        None => {
            if let Some(addr) = data
                .dht_node
                .lock()
                .unwrap()
                .get(&get_dht_key(witness_id.as_bytes()))
            {
                let url = format!("http://{}/witness_ips/{}", addr, witness_id);
                log::info!("Found witness IP for {:?} at {:?}", witness_id, url);
                if let Ok(resp) = reqwest::get(url).await {
                    let body = resp.text().await.unwrap();
                    let witness_ip: WitnessIp = serde_json::from_str(&body).unwrap();
                    return HttpResponse::Ok().json(witness_ip);
                };
            };
        }
    }
    HttpResponse::NotFound().body(format!("Witness IP for {:?} not found", witness_id))
}

#[actix_web::put("/witness_ips/{witness_id}")]
async fn witness_ip_put(
    path: web::Path<String>,
    body: web::Json<WitnessIp>,
    data: web::Data<AppState>,
) -> impl Responder {
    let witness_id = &*path;
    let witness_ip = &*body;
    let resp = {
        let mut witness_ips = data.witness_ips.lock().unwrap();
        log::info!(
            "Saving {data:?} for witness {id:?}",
            id = witness_id,
            data = witness_ip
        );
        match witness_ips.insert(witness_id.clone(), witness_ip.clone()) {
            Some(_) => HttpResponse::Ok().json(witness_ip),
            None => HttpResponse::Created().json(witness_ip),
        }
    };
    {
        let mut node = data.dht_node.lock().unwrap();
        node.insert(
            get_dht_key(witness_id.as_bytes()),
            &data.api_public_addr.to_string(),
        );
    };
    resp
}

pub(crate) fn get_dht_key(value: &[u8]) -> Key {
    let mut hasher = Sha3_256::default();
    hasher.update(value);
    Key(hasher.finalize()[..].try_into().unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let Opts {
        api_port,
        api_public_host,
        db_path,
        dht_port,
        dht_bootstrap_addr,
    } = Opts::from_args();

    let api_public_addr = ToSocketAddrs::to_socket_addrs(&(api_public_host, api_port))
        .expect("Invalid public API address")
        .next()
        .unwrap();
    log::info!("Public API address is {:?}", api_public_addr);

    let dht_addr = SocketAddr::from(([0, 0, 0, 0], dht_port));
    log::info!("Starting DHT peer at {:?}", dht_addr);

    let dht_bootstrap_addr = dht_bootstrap_addr.and_then(|addr| {
        let addrs = ToSocketAddrs::to_socket_addrs(&addr).expect("Invalid bootstrap address");
        let addr = addrs.into_iter().next()?;
        log::info!("Bootstrapping DHT from {:?}", addr);
        Some(addr)
    });

    let dht_node = Node::new(
        &dht_addr.ip().to_string(),
        &dht_addr.port().to_string(),
        dht_bootstrap_addr.map(|addr| NodeData {
            addr: addr.to_string(),
            id: Key::new(random()),
        }),
    );

    let db = Arc::new(SledEventDatabase::new(db_path.as_path()).unwrap());
    let event_processor = Mutex::new(EventStorage::new(Arc::clone(&db)));

    let state = web::Data::new(AppState {
        dht_node: Mutex::new(dht_node),
        api_public_addr,
        event_processor,
        witness_ips: Mutex::new(HashMap::new()),
    });

    let api_addr = SocketAddr::from(([0, 0, 0, 0], api_port));
    log::info!("Starting API server at {:?}", api_addr);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(key_state_get)
            .service(key_log_get)
            .service(message_put)
            .service(witness_ip_get)
            .service(witness_ip_put)
    })
    .bind(api_addr)?
    .run()
    .await
}

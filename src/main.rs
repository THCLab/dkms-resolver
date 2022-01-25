use std::{
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs},
    sync::Mutex,
};

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use kademlia_dht::{Key, Node, NodeData};
use rand::random;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha3::{Digest, Sha3_256};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {
    /// API listen port.
    #[structopt(long, env, default_value = "9599")]
    api_port: u16,

    /// DHT listen port.
    #[structopt(long, env, default_value = "9145")]
    dht_port: u16,

    /// DHT bootstrap IP address.
    #[structopt(long, env)]
    bootstrap_addr: Option<String>,
}

struct AppState {
    dht_node: Mutex<Node>,
    pub_api_addr: SocketAddr,
    key_states: Mutex<HashMap<String, Value>>,
    witness_ips: Mutex<HashMap<String, WitnessIp>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WitnessIp {
    ip: SocketAddr,
}

#[actix_web::get("/key_states/{issuer_id}")]
async fn key_state_get(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let issuer_id = &*path;
    let key_states = data.key_states.lock().unwrap();
    match key_states.get(issuer_id) {
        Some(key_state) => return HttpResponse::Ok().json(key_state),
        None => {
            if let Some(addr) = data
                .dht_node
                .lock()
                .unwrap()
                .get(&get_dht_key(issuer_id.as_bytes()))
            {
                let url = format!("http://{}/key_states/{}", addr, issuer_id);
                let resp = reqwest::get(url).await.unwrap();
                let body = resp.text().await.unwrap();
                let key_state: Value = serde_json::from_str(&body).unwrap();
                return HttpResponse::Ok().json(key_state);
            };
        }
    }
    return HttpResponse::NotFound().body(format!("Key state for {:?} not found", issuer_id));
}

#[actix_web::put("/key_states/{issuer_id}")]
async fn key_state_put(
    path: web::Path<String>,
    body: web::Json<Value>,
    data: web::Data<AppState>,
) -> impl Responder {
    let issuer_id = &*path;
    let key_state = &*body;
    let resp = {
        let mut key_states = data.key_states.lock().unwrap();
        log::info!(
            "Saving {data:?} for issuer {id:?}",
            id = issuer_id,
            data = key_state
        );
        match key_states.insert(issuer_id.clone(), key_state.clone()) {
            Some(_) => HttpResponse::Ok().json(key_state),
            None => HttpResponse::Created().json(key_state),
        }
    };
    {
        let mut node = data.dht_node.lock().unwrap();
        node.insert(
            get_dht_key(issuer_id.as_bytes()),
            &data.pub_api_addr.to_string(),
        );
    };
    resp
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
                let resp = reqwest::get(url).await.unwrap();
                let body = resp.text().await.unwrap();
                let witness_ip: WitnessIp = serde_json::from_str(&body).unwrap();
                return HttpResponse::Ok().json(witness_ip);
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
            &data.pub_api_addr.to_string(),
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
        dht_port,
        bootstrap_addr,
    } = Opts::from_args();

    let pub_ip = match public_ip::addr().await {
        Some(addr) => addr,
        None => {
            panic!("Can't resolve public IP")
        }
    };
    let pub_api_addr = SocketAddr::from((pub_ip, api_port));
    log::info!("Resolved public API address as {:?}", pub_api_addr);

    let dht_addr = SocketAddr::from(([0, 0, 0, 0], dht_port));
    log::info!("Starting DHT peer at {:?}", dht_addr);

    let bootstrap_addr = bootstrap_addr.and_then(|addr| {
        let addrs = ToSocketAddrs::to_socket_addrs(&addr).expect("Invalid bootstrap address");
        let addr = addrs.into_iter().next()?;
        log::info!("Bootstrapping DHT from {:?}", addr);
        Some(addr)
    });

    let dht_node = Node::new(
        &dht_addr.ip().to_string(),
        &dht_addr.port().to_string(),
        bootstrap_addr.map(|addr| NodeData {
            addr: addr.to_string(),
            id: Key::new(random()),
        }),
    );

    let state = web::Data::new(AppState {
        dht_node: Mutex::new(dht_node),
        pub_api_addr,
        key_states: Mutex::new(HashMap::new()),
        witness_ips: Mutex::new(HashMap::new()),
    });

    let api_addr = SocketAddr::from(([0, 0, 0, 0], api_port));
    log::info!("Starting API server at {:?}", api_addr);
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(key_state_get)
            .service(key_state_put)
            .service(witness_ip_get)
            .service(witness_ip_put)
    })
    .bind(api_addr)?
    .run()
    .await
}

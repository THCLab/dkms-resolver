use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Debug, Default)]
struct AppState {
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
        Some(key_state) => HttpResponse::Ok().json(key_state),
        None => HttpResponse::NotFound().body(format!("Key state for {:?} not found", issuer_id)),
    }
}

#[actix_web::put("/key_states/{issuer_id}")]
async fn key_state_put(
    path: web::Path<String>,
    body: web::Json<Value>,
    data: web::Data<AppState>,
) -> impl Responder {
    let issuer_id = &*path;
    let key_state = &*body;
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
}

#[actix_web::get("/witness_ips/{witness_id}")]
async fn witness_ip_get(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let witness_id = &*path;
    let witness_ips = data.witness_ips.lock().unwrap();
    match witness_ips.get(witness_id) {
        Some(witness_ip) => HttpResponse::Ok().json(witness_ip),
        None => HttpResponse::NotFound().body(format!("Witness IP for {:?} not found", witness_id)),
    }
}

#[actix_web::put("/witness_ips/{witness_id}")]
async fn witness_ip_put(
    path: web::Path<String>,
    body: web::Json<WitnessIp>,
    data: web::Data<AppState>,
) -> impl Responder {
    let witness_id = &*path;
    let witness_ip = &*body;
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

    let api_addr = SocketAddr::from(([127, 0, 0, 1], api_port));

    log::info!("Starting API server at {:?}", api_addr);

    let state = web::Data::new(AppState::default());

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

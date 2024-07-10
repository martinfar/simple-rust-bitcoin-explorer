use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use bitcoin::BlockHash;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use log::{info, error, debug};
use reqwest::StatusCode;
use std::fs::File;
use std::io::Read;


#[derive(Debug, Deserialize,Clone)]
struct Config {
    rpc: RpcConfig,
    server: ServerConfig,
}

#[derive(Debug, Deserialize,Clone)]
struct RpcConfig {
    url: String,
    user: String,
    pass: String,
}


#[derive(Debug, Deserialize,Clone)]
struct ServerConfig {
    host: String,
    port: u16,
}


fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open("config.yaml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_yaml::from_str(&contents)?;
    Ok(config)
}

#[derive(Serialize, Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    id: String,
    method: String,
    params: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct RpcResponse {
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

async fn make_rpc_call(method: &str, params: Vec<serde_json::Value>, config: &RpcConfig) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let rpc_request = RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: "1".to_string(),
        method: method.to_string(),
        params: params.clone(),
    };

    info!("Making RPC call: method={}, params={:?}", method, params);

    let response = match client.post(&config.url)
        .basic_auth(&config.user, Some(&config.pass))
        .json(&rpc_request)
        .send()
        .await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to connect to RPC server: {}", e);
            return Err(format!("RPC connection error: {}", e).into());
        }
    };

    let status = response.status();
    debug!("RPC response status: {}", status);

    if status != StatusCode::OK {
        let error_body = response.text().await.unwrap_or_else(|_| "Unable to read error response".to_string());
        error!("RPC server returned non-OK status. Status: {}, Body: {}", status, error_body);
        return Err(format!("RPC server error. Status: {}", status).into());
    }

    let response_body = match response.text().await {
        Ok(body) => {
            debug!("Raw RPC response: {}", body);
            body
        },
        Err(e) => {
            error!("Failed to read RPC response body: {}", e);
            return Err(format!("Failed to read RPC response: {}", e).into());
        }
    };

    let rpc_response: RpcResponse = match serde_json::from_str(&response_body) {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to parse RPC response: {}. Raw response: {}", e, response_body);
            return Err(format!("Failed to parse RPC response: {}", e).into());
        }
    };

    match rpc_response.result {
        Some(result) => {
            info!("RPC call successful: method={}", method);
            Ok(result)
        },
        None => {
            let error_msg = rpc_response.error
                .map(|e| format!("{:?}", e))
                .unwrap_or_else(|| "Unknown error".to_string());
            error!("RPC call failed: method={}, error={}", method, error_msg);
            Err(error_msg.into())
        }
    }
}

async fn get_block_info(block_hash: web::Path<String>, config: web::Data<RpcConfig>) -> impl Responder {
    let hash = match BlockHash::from_str(&block_hash) {
        Ok(h) => h,
        Err(_) => return HttpResponse::BadRequest().body("Invalid block hash"),
    };

    match make_rpc_call("getblock", vec![serde_json::to_value(hash.to_string()).unwrap()], &config).await {
        Ok(block_info) => HttpResponse::Ok().json(block_info),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve block information"),
    }
}

async fn get_transaction_info(tx_id: web::Path<String>, config: web::Data<RpcConfig>) -> impl Responder {
    match make_rpc_call("getrawtransaction", vec![serde_json::to_value(tx_id.to_string()).unwrap(), serde_json::to_value(true).unwrap()], &config).await {
        Ok(tx_info) => HttpResponse::Ok().json(tx_info),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve transaction information"),
    }
}

async fn get_latest_blocks(config: web::Data<RpcConfig>) -> impl Responder {
    match make_rpc_call("getblockcount", vec![], &config).await {
        Ok(block_count) => {
            let block_count: i64 = serde_json::from_value(block_count).unwrap();
            let mut latest_blocks = Vec::new();

            for i in 0..10 {
                if let Ok(block_hash) = make_rpc_call("getblockhash", vec![serde_json::to_value(block_count - i).unwrap()], &config).await {
                    if let Ok(block_info) = make_rpc_call("getblock", vec![block_hash], &config).await {
                        latest_blocks.push(block_info);
                    }
                }
            }

            HttpResponse::Ok().json(latest_blocks)
        },
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve latest blocks"),
    }
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {

    env_logger::init();

    info!("Starting Bitcoin Block Explorer API");
    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Configuration error"));
        }
    };

    let rpc_config = web::Data::new(config.rpc.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(rpc_config.clone())
            .route("/block/{hash}", web::get().to(get_block_info))
            .route("/tx/{txid}", web::get().to(get_transaction_info))
            .route("/latest_blocks", web::get().to(get_latest_blocks))
    })
        .bind(format!("{}:{}", config.server.host, config.server.port))?
        .run()
        .await
}

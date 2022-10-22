/*
 * Filename: main.rs
 * Created Date: Tuesday, October 18th 2022, 5:15:15 pm
 * Author: Jonathan Haws
 *
 * Copyright (c) 2022 WiTricity
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use crate::manager::NetworkManager;
use serde::{Deserialize, Serialize};
use std::vec::Vec;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use warp::{http, Filter};

pub mod endpoint;
pub mod manager;
pub mod network;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ErrorResponse {
    name: String,
    msg: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
struct HandshakeResponse {
    Implements: Vec<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
struct SetCapabilityResponse {
    Scope: String,
    ConnectivityScope: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
struct JoinResponse {
    InterfaceName: network::JoinResponse,
}

async fn api_plugin_activate(payload: bytes::Bytes) -> Result<impl warp::Reply, warp::Rejection> {
    log_body(&payload);
    let rsp = HandshakeResponse {
        Implements: vec![String::from("NetworkDriver")],
    };

    let mut status: http::StatusCode = http::StatusCode::OK;
    let jrsp = match serde_json::to_string(&rsp) {
        Ok(jrsp) => jrsp,
        Err(_) => {
            status = http::StatusCode::BAD_REQUEST;
            String::from(r#"{"Err":"Serializing response to Plugin.Activate"}"#)
        }
    };
    println!("Plugin.Activate: {}", jrsp);
    Ok(warp::reply::with_status(jrsp, status))
}

async fn api_get_capabilities(payload: bytes::Bytes) -> Result<impl warp::Reply, warp::Rejection> {
    log_body(&payload);
    let rsp = SetCapabilityResponse {
        Scope: String::from("local"),
        ConnectivityScope: String::from("local"),
    };

    let mut status: http::StatusCode = http::StatusCode::OK;
    let jrsp = match serde_json::to_string(&rsp) {
        Ok(jrsp) => jrsp,
        Err(_) => {
            status = http::StatusCode::BAD_REQUEST;
            String::from(r#"{"Err":"Serializing response to NetworkDriver.GetCapabilities"}"#)
        }
    };

    println!("NetworkDriver.GetCapabilities: {}", jrsp);
    Ok(warp::reply::with_status(jrsp, status))
}

async fn api_network_create(
    payload: bytes::Bytes,
    mgr: NetworkManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    log_body(&payload);

    let mut status: http::StatusCode = http::StatusCode::OK;
    let reply = match serde_json::from_slice::<serde_json::Value>(&payload) {
        Ok(v) => {
            let mut error = false;
            let uid = match v["NetworkID"].as_str() {
                Some(u) => u.to_string(),
                None => {
                    println!("Error parsing network ID: {}", v["NetworkID"]);
                    error = true;
                    String::new()
                }
            };
            let opt = match v["Options"]["com.docker.network.generic"].as_str() {
                Some(o) => o.to_string(),
                None => v["Options"]["com.docker.network.generic"].to_string(),
            };
            if !error {
                mgr.network_create(uid, opt);
                "{}"
            } else {
                status = http::StatusCode::BAD_REQUEST;
                r#"{"Err":"Invalid network ID"}"#
            }
        }
        Err(_) => r#"{"Err":"Unable to parse JSON payload"}"#,
    };

    println!("NetworkDriver.CreateNetwork: {}", reply);
    Ok(warp::reply::with_status(reply, status))
}

async fn api_network_delete(
    payload: bytes::Bytes,
    mgr: NetworkManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    log_body(&payload);

    let mut status: http::StatusCode = http::StatusCode::OK;
    let reply = match serde_json::from_slice::<serde_json::Value>(&payload) {
        Ok(v) => {
            let mut error = false;
            let uid = match v["NetworkID"].as_str() {
                Some(u) => u.to_string(),
                None => {
                    println!("Error parsing network ID: {}", v["NetworkID"]);
                    error = true;
                    String::new()
                }
            };
            if !error {
                mgr.network_delete(uid);
                "{}"
            } else {
                status = http::StatusCode::BAD_REQUEST;
                r#"{"Err":"Invalid network ID"}"#
            }
        }
        Err(_) => r#"{"Err":"Unable to parse JSON payload"}"#,
    };

    println!("NetworkDriver.DeleteNetwork: {}", reply);
    Ok(warp::reply::with_status(reply, status))
}

async fn api_endpoint_create(
    payload: bytes::Bytes,
    mgr: NetworkManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    log_body(&payload);

    let mut status: http::StatusCode = http::StatusCode::OK;
    let reply = match serde_json::from_slice::<serde_json::Value>(&payload) {
        Ok(v) => {
            let mut error = false;
            let nuid = match v["NetworkID"].as_str() {
                Some(u) => u.to_string(),
                None => {
                    println!("Error parsing network ID: {}", v["NetworkID"]);
                    error = true;
                    String::new()
                }
            };
            let epuid = match v["EndpointID"].as_str() {
                Some(u) => u.to_string(),
                None => {
                    println!("Error parsing endpoint ID: {}", v["EndpointID"]);
                    error = true;
                    String::new()
                }
            };
            if !error {
                mgr.endpoint_create(nuid, epuid);
                "{}"
            } else {
                status = http::StatusCode::BAD_REQUEST;
                r#"{"Err":"Invalid network ID or endpoint ID"}"#
            }
        }
        Err(_) => r#"{"Err":"Unable to parse JSON payload"}"#,
    };

    println!("NetworkDriver.CreateEndpoint: {}", reply);
    Ok(warp::reply::with_status(reply, status))
}

async fn api_endpoint_delete(
    payload: bytes::Bytes,
    mgr: NetworkManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    log_body(&payload);

    let mut status: http::StatusCode = http::StatusCode::OK;
    let reply = match serde_json::from_slice::<serde_json::Value>(&payload) {
        Ok(v) => {
            let mut error = false;
            let nuid = match v["NetworkID"].as_str() {
                Some(u) => u.to_string(),
                None => {
                    println!("Error parsing network ID: {}", v["NetworkID"]);
                    error = true;
                    String::new()
                }
            };
            let epuid = match v["EndpointID"].as_str() {
                Some(u) => u.to_string(),
                None => {
                    println!("Error parsing endpoint ID: {}", v["EndpointID"]);
                    error = true;
                    String::new()
                }
            };
            if !error {
                mgr.endpoint_delete(nuid, epuid);
                "{}"
            } else {
                status = http::StatusCode::BAD_REQUEST;
                r#"{"Err":"Invalid network ID or endpoint ID"}"#
            }
        }
        Err(_) => r#"{"Err":"Unable to parse JSON payload"}"#,
    };

    println!("NetworkDriver.DeleteEndpoint: {}", reply);
    Ok(warp::reply::with_status(reply, status))
}

async fn api_endpoint_info(
    payload: bytes::Bytes,
    _mgr: NetworkManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    log_body(&payload);

    // ! TODO Add error handling
    Ok(warp::reply::with_status("{}", http::StatusCode::OK))
}

async fn api_network_join(
    payload: bytes::Bytes,
    mgr: NetworkManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    log_body(&payload);

    let mut status: http::StatusCode = http::StatusCode::OK;
    let reply = match serde_json::from_slice::<serde_json::Value>(&payload) {
        Ok(v) => {
            let mut error = false;
            let nuid = match v["NetworkID"].as_str() {
                Some(u) => u.to_string(),
                None => {
                    println!("Error parsing network ID: {}", v["NetworkID"]);
                    error = true;
                    String::new()
                }
            };
            let epuid = match v["EndpointID"].as_str() {
                Some(u) => u.to_string(),
                None => {
                    println!("Error parsing endpoint ID: {}", v["EndpointID"]);
                    error = true;
                    String::new()
                }
            };
            let sbox = match v["SandboxKey"].as_str() {
                Some(u) => u.to_string(),
                None => {
                    println!("Error parsing sandbox key: {}", v["SandboxKey"]);
                    error = true;
                    String::new()
                }
            };
            let opt = match v["Options"].as_str() {
                Some(o) => o.to_string(),
                None => v["Options"].to_string(),
            };
            if !error {
                match mgr.endpoint_attach(nuid, epuid, sbox, opt) {
                    Ok(joinrsp) => {
                        let rsp = JoinResponse {
                            InterfaceName: joinrsp,
                        };
                        match serde_json::to_string(&rsp) {
                            Ok(jrsp) => jrsp,
                            Err(_) => String::from(
                                r#"{"Err":"Serializing response to NetworkDriver.Join"}"#,
                            ),
                        }
                    }
                    Err(_) => String::from(r#"{"Err":"Error attaching endpoint to network"}"#),
                }
            } else {
                status = http::StatusCode::BAD_REQUEST;
                String::from(r#"{"Err":"Invalid network ID, endpoint ID, or sandbox key"}"#)
            }
        }
        Err(_) => String::from(r#"{"Err":"Unable to parse JSON payload"}"#),
    };

    println!("NetworkDriver.Join: {}", reply);
    Ok(warp::reply::with_status(reply, status))
}

async fn api_network_leave(
    payload: bytes::Bytes,
    mgr: NetworkManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    log_body(&payload);

    let mut status: http::StatusCode = http::StatusCode::OK;
    let reply = match serde_json::from_slice::<serde_json::Value>(&payload) {
        Ok(v) => {
            let mut error = false;
            let nuid = match v["NetworkID"].as_str() {
                Some(u) => u.to_string(),
                None => {
                    println!("Error parsing network ID: {}", v["NetworkID"]);
                    error = true;
                    String::new()
                }
            };
            let epuid = match v["EndpointID"].as_str() {
                Some(u) => u.to_string(),
                None => {
                    println!("Error parsing endpoint ID: {}", v["EndpointID"]);
                    error = true;
                    String::new()
                }
            };
            if !error {
                mgr.endpoint_detach(nuid, epuid);
                "{}"
            } else {
                status = http::StatusCode::BAD_REQUEST;
                r#"{"Err":"Invalid network ID or endpoint ID"}"#
            }
        }
        Err(_) => r#"{"Err":"Unable to parse JSON payload"}"#,
    };

    println!("NetworkDriver.Leave: {}", reply);
    Ok(warp::reply::with_status(reply, status))
}

async fn api_discover_new(
    payload: bytes::Bytes,
    _mgr: NetworkManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    log_body(&payload);

    // ! TODO Add error handling
    Ok(warp::reply::with_status("{}", http::StatusCode::OK))
}

async fn api_discover_delete(
    payload: bytes::Bytes,
    _mgr: NetworkManager,
) -> Result<impl warp::Reply, warp::Rejection> {
    log_body(&payload);

    // ! TODO Add error handling
    Ok(warp::reply::with_status("{}", http::StatusCode::OK))
}

fn log_body(_payload: &bytes::Bytes) {
    // println!(
    //     "Request body: {}",
    //     std::str::from_utf8(&payload).expect("error converting bytes to &str")
    // );
}

fn process_body() -> impl Filter<Extract = (bytes::Bytes,), Error = warp::Rejection> + Copy {
    warp::body::content_length_limit(1024 * 16).and(warp::body::bytes())
}

#[tokio::main]
async fn main() {
    let mgr = NetworkManager::new();
    mgr.network_load().await;
    let filter = warp::any().map(move || mgr.clone());

    let payload = warp::post()
        .and(warp::path("Plugin.Activate"))
        .and(warp::path::end())
        .and(process_body())
        .and_then(api_plugin_activate);

    let get_cap = warp::post()
        .and(warp::path("NetworkDriver.GetCapabilities"))
        .and(warp::path::end())
        .and(process_body())
        .and_then(api_get_capabilities);

    let nw_create = warp::post()
        .and(warp::path("NetworkDriver.CreateNetwork"))
        .and(warp::path::end())
        .and(process_body())
        .and(filter.clone())
        .and_then(api_network_create);

    let nw_del = warp::post()
        .and(warp::path("NetworkDriver.DeleteNetwork"))
        .and(warp::path::end())
        .and(process_body())
        .and(filter.clone())
        .and_then(api_network_delete);

    let endp_create = warp::post()
        .and(warp::path("NetworkDriver.CreateEndpoint"))
        .and(warp::path::end())
        .and(process_body())
        .and(filter.clone())
        .and_then(api_endpoint_create);

    let endp_del = warp::post()
        .and(warp::path("NetworkDriver.DeleteEndpoint"))
        .and(warp::path::end())
        .and(process_body())
        .and(filter.clone())
        .and_then(api_endpoint_delete);

    let endp_info = warp::post()
        .and(warp::path("NetworkDriver.EndpointOperInfo"))
        .and(warp::path::end())
        .and(process_body())
        .and(filter.clone())
        .and_then(api_endpoint_info);

    let nw_join = warp::post()
        .and(warp::path("NetworkDriver.Join"))
        .and(warp::path::end())
        .and(process_body())
        .and(filter.clone())
        .and_then(api_network_join);

    let nw_leave = warp::post()
        .and(warp::path("NetworkDriver.Leave"))
        .and(warp::path::end())
        .and(process_body())
        .and(filter.clone())
        .and_then(api_network_leave);

    let dsc_new = warp::post()
        .and(warp::path("NetworkDriver.DiscoverNew"))
        .and(warp::path::end())
        .and(process_body())
        .and(filter.clone())
        .and_then(api_discover_new);

    let dsc_del = warp::post()
        .and(warp::path("NetworkDriver.DiscoverDelete"))
        .and(warp::path::end())
        .and(process_body())
        .and(filter.clone())
        .and_then(api_discover_delete);

    let routes = payload
        .or(get_cap)
        .or(nw_create)
        .or(nw_del)
        .or(endp_create)
        .or(endp_del)
        .or(endp_info)
        .or(nw_join)
        .or(nw_leave)
        .or(dsc_new)
        .or(dsc_del);

    let incoming =
        UnixListenerStream::new(UnixListener::bind("/run/docker/plugins/rustyvxcan.sock").unwrap());
    warp::serve(routes).run_incoming(incoming).await;

    // Alternatively, run an IP-based plugin
    // Create the file /etc/docker/plugins/rustyvxcan.json and add the following:
    //      {
    //          "Name": "rustyvxcan",
    //          "Addr": "http://127.0.0.1:7373"
    //      }
    //
    // Then, uncomment the following line (and remove the UnixListener above)
    // warp::serve(routes).run(([127,0,0,1],7373)).await;
}

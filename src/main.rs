// promdoc -- Prometheus Documentation Tool
// Copyright (C) 2023, Tony Rippy
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[macro_use]
extern crate log;

use bytes::Bytes;
use clap::Parser;
use env_logger::Env;
use http_body_util::Full;
use hyper::{server::conn::http1, service::service_fn, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use std::io::Error;
use std::process::ExitCode;
use tokio::signal;
use tokio::{net::TcpListener, runtime, task};

const INDEX_HTML: &str = include_str!("../ui/dist/index.html");
const INDEX_JS: &str = include_str!("../ui/dist/js/index.min.js");

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(short, long, default_value_t = 9095)]
    port: u16,
}

#[derive(Serialize, Deserialize)]
struct ClientConfig {
    prometheus_urls: Vec<String>,
}

async fn serve<R>(req: Request<R>) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    match req.uri().path() {
        "/" => Response::builder()
            .header("Content-Type", "text/html; charset=utf-8")
            .status(StatusCode::OK)
            .body(INDEX_HTML.into()),
        "/js" => Response::builder()
            .header("Content-Type", "text/javascript; charset=utf-8")
            .status(StatusCode::OK)
            .body(INDEX_JS.into()),
        "/config" => {
            match serde_json::to_string(&ClientConfig {
                prometheus_urls: vec!["http://localhost:9090".to_string()],
            }) {
                Ok(json) => Response::builder()
                    .header("Content-Type", "application/json; charset=utf-8")
                    .status(StatusCode::OK)
                    .body(json.into()),
                Err(err) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(err.to_string().into()),
            }
        }
        "/-/healthy" => Response::builder().status(StatusCode::OK).body("OK".into()),
        "/-/ready" => Response::builder().status(StatusCode::OK).body("OK".into()),
        "/-/reload" => Response::builder()
            .status(StatusCode::NOT_IMPLEMENTED)
            .body(Full::default()),
        "/-/quit" => Response::builder()
            .status(StatusCode::NOT_IMPLEMENTED)
            .body(Full::default()),
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::default()),
    }
}

async fn serving_loop(args: &Args) -> Result<(), Error> {
    let listener = TcpListener::bind((args.host.as_str(), args.port)).await?;
    info!("Listening on port {}:{}", args.host, args.port);
    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Interrupt signal received.");
                break
            }
            Ok((tcp_stream, _)) = listener.accept() => {
                tokio::spawn(
                    http1::Builder::new()
                        .keep_alive(false)
                        .serve_connection(TokioIo::new(tcp_stream), service_fn(serve)));
            }
        }
        task::yield_now().await;
    }
    Ok(())
}

fn main() -> ExitCode {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    match runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .and_then(|rt| rt.block_on(serving_loop(&args)))
    {
        Err(err) => {
            error!("{}", err);
            ExitCode::FAILURE
        }
        _ => ExitCode::SUCCESS,
    }
}

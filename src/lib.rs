#![allow(warnings)]
use anyhow::{anyhow, Result};
use hyper::header::CONTENT_TYPE;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::Encoder;
use prometheus::TextEncoder;
use std::convert::Infallible;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use tokio::task::JoinHandle;

use tracing::{error, info};

async fn metrics(req: Request<Body>, state: Arc<AppState>) -> Result<Response<Body>, hyper::Error> {
    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            let mut buffer = vec![];
            let encoder = TextEncoder::new();
            let metric_families = state.exporter.registry().gather();
            encoder.encode(&metric_families, &mut buffer).unwrap();

            Response::builder()
                .status(200)
                .header(CONTENT_TYPE, encoder.format_type())
                .body(Body::from(buffer))
                .unwrap()
        }
        _ => Response::builder()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap(),
    };

    Ok(response)
}

struct AppState {
    exporter: PrometheusExporter,
}

pub fn init() -> Result<(SocketAddr, JoinHandle<Result<()>>)> {
    let addr = ([0, 0, 0, 0], 9090).into();

    let handle = tokio::spawn(async move {
        let exporter = match opentelemetry_prometheus::exporter().try_init() {
            Ok(exporter) => exporter,
            Err(err) => {
                return Err(anyhow!(
                    "Failed to creat prometheus serve metrics {:?}",
                    err
                ))
            }
        };

        let state = Arc::new(AppState { exporter });

        // For every connection, we must make a `Service` to handle all
        // incoming HTTP requests on said connection.
        let make_svc = make_service_fn(move |_conn| {
            let state = state.clone();
            // This is the `Service` that will handle the connection.
            // `service_fn` is a helper to convert a function that
            // returns a Response into a `Service`.
            async move { Ok::<_, Infallible>(service_fn(move |req| metrics(req, state.clone()))) }
        });

        let server = Server::bind(&addr).serve(make_svc);

        info!("Serving prometheus metrics on http://{}", addr);

        server.await.map_err(Into::into)
    });

    Ok((addr, handle))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

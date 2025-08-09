use std::{convert::Infallible, error::Error, net::SocketAddr, time::Duration};

use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, Response, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::{
    client::legacy::{Client, connect::HttpConnector},
    rt::TokioIo,
};
use tokio::net::TcpListener;

use crate::core::cache::mem_cache::{CacheEntry, MemoryCache};

pub struct ProxyServer;

impl ProxyServer {
    pub async fn start(port: u16, origin: String) -> Result<(), Box<dyn Error>> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr).await?;

        let cache = MemoryCache::new(Duration::from_secs(60), 100);

        println!("Caching proxy server started on http://127.0.0.1:{port}");
        println!("Forwarding requests to: {origin}");

        let client = Client::builder(TokioExecutor).build(HttpConnector::new());

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            let origin = origin.clone();
            let client = client.clone();
            let cache = cache.clone();

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(move |req| {
                            let origin = origin.clone();
                            let client = client.clone();
                            let cache = cache.clone();
                            async move { ProxyServer::transform(req, origin, client, cache).await }
                        }),
                    )
                    .await
                {
                    eprintln!("Error serving connection: {err}");
                }
            });
        }
    }

    async fn transform(
        request: Request<hyper::body::Incoming>,
        origin: String,
        client: Client<HttpConnector, hyper::body::Incoming>,
        cache: MemoryCache,
    ) -> Result<Response<Full<Bytes>>, Infallible> {
        let path_and_query = request
            .uri()
            .path_and_query()
            .map(|x| x.as_str())
            .unwrap_or("/");

        let target_url = format!("{origin}{path_and_query}");

        let cache_key = if request.method() == Method::GET {
            target_url.clone()
        } else {
            String::new()
        };

        if !cache_key.is_empty() {
            if let Some(cached_entry) = cache.get(&cache_key) {
                println!("Cache HIT for: {target_url}");

                let response_builder = Response::builder()
                    .status(cached_entry.status_code)
                    .header("X-Cache", "HIT");

                return Ok(response_builder
                    .body(Full::new(cached_entry.response_body))
                    .unwrap());
            }
        }

        println!("Cache MISS for: {target_url}");

        let uri = match target_url.parse::<hyper::Uri>() {
            Ok(uri) => uri,
            Err(e) => {
                eprintln!("Failed to parse URL {target_url}: {e}");
                return Ok(Response::builder()
                    .status(500)
                    .header("X-Cache", "MISS")
                    .body(Full::new(Bytes::from("Internal Server Error: Invalid URL")))
                    .unwrap());
            }
        };

        let forwarded_request = Request::builder()
            .method(request.method())
            .uri(uri)
            .body(request.into_body())
            .unwrap();

        match client.request(forwarded_request).await {
            Ok(response) => {
                let status = response.status();
                let headers = response.headers().clone();

                match response.into_body().collect().await {
                    Ok(collected) => {
                        let body_bytes = collected.to_bytes();

                        if !cache_key.is_empty() && status.is_success() {
                            let cache_entry =
                                CacheEntry::new(body_bytes.clone(), status, headers.clone());
                            cache.set(cache_key, cache_entry);
                        }

                        let response_builder =
                            Response::builder().status(status).header("X-Cache", "MISS");

                        Ok(response_builder.body(Full::new(body_bytes)).unwrap())
                    }
                    Err(e) => {
                        eprintln!("Failed to read response body: {e}");
                        Ok(Response::builder()
                            .status(502)
                            .header("X-Cache", "MISS")
                            .body(Full::new(Bytes::from(
                                "Bad Gateway: Failed to read response",
                            )))
                            .unwrap())
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to forward request: {e}");
                Ok(Response::builder()
                    .status(502)
                    .header("X-Cache", "MISS")
                    .body(Full::new(Bytes::from(
                        "Bad Gateway: Failed to connect to origin server",
                    )))
                    .unwrap())
            }
        }
    }
}

#[derive(Clone)]
struct TokioExecutor;

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}

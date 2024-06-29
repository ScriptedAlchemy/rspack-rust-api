#![deny(warnings)]

use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use tokio::net::TcpListener;
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::time::Instant;
use url::form_urlencoded;
mod edge_compile;
mod memory_fs;
// An async function that consumes a request, executes the rspack file, and returns a response.
async fn handle_request(req: Request<impl hyper::body::Body>) -> Result<Response<Full<Bytes>>, Infallible> {
    if req.uri().path() == "/favicon.ico" {
        return Ok(Response::new(Full::new(Bytes::new())));
    }

    let start_time = Instant::now();

    // Parse the query parameters
    let query_params: HashMap<_, _> = req.uri().query().map(|v| {
        form_urlencoded::parse(v.as_bytes()).into_iter().collect()
    }).unwrap_or_else(HashMap::new);
    // Log the query parameters for debugging
    dbg!(req.uri().clone());
    // Get the entry parameter
    let entry = query_params.get("entry").cloned().unwrap_or_else(|| "".to_string().into());
    // Pass the entry parameter to the compile function
    edge_compile::compile(Some(entry.clone().to_string())).await;
    let duration = start_time.elapsed();

    let response_body = format!("Compile time: {:?}", duration);

    Ok(Response::new(Full::new(Bytes::from(response_body))))
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

    // This address is localhost
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    // Bind to the port and listen for incoming TCP connections
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        // When an incoming TCP connection is received grab a TCP stream for
        // client<->server communication.
        //
        // Note, this is a .await point, this loop will loop forever but is not a busy loop. The
        // .await point allows the Tokio runtime to pull the task off of the thread until the task
        // has work to do. In this case, a connection arrives on the port we are listening on and
        // the task is woken up, at which point the task is then put back on a thread, and is
        // driven forward by the runtime, eventually yielding a TCP stream.
        let (tcp, _) = listener.accept().await?;
        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(tcp);

        // Spin up a new task in Tokio so we can continue to listen for new TCP connection on the
        // current task without waiting for the processing of the HTTP1 connection we just received
        // to finish
        tokio::task::spawn(async move {
            // Handle the connection from the client using HTTP1 and pass any
            // HTTP requests received on that connection to the `handle_request` function
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle_request))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

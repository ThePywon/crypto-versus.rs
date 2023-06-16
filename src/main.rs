use std::convert::Infallible;
use std::io::Read;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
mod api_endpoints;
use api_endpoints::get_endpoints;
use dotenvy;
mod database;
mod hash;
use std::fs::File;

#[tokio::main]
async fn main() {
  dotenvy::dotenv().expect("Could not load environment variables.");

  let addr = SocketAddr::from(([127, 0, 0, 1], std::env::var("PORT").unwrap().parse().unwrap()));

  let make_svc = make_service_fn(|_conn| async {
    Ok::<_, Infallible>(service_fn(req_handler))
  });

  let server = Server::bind(&addr).serve(make_svc);

  if let Err(e) = server.await {
    eprintln!("Server error: {}", e);
  }
}

async fn req_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  dbg!(&req);
  let path = req.uri().path();

  if path.starts_with("/api/") {
    if let Some(endpoint) = get_endpoints().get(&(req.method(), &path[5..])) {
      return endpoint(req).await
    }
  }
  else {
    let mut public_path = String::from("./src/public") + path;
    if !public_path.ends_with(".html") {public_path += ".html"}

    if let Ok(mut file) = File::open(public_path) {
      if file.metadata().unwrap().is_file() {
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        return Ok(Response::builder().body(Body::from(content)).unwrap())
      }
    }
  }

  Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()).unwrap())
}

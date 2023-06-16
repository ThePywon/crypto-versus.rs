use std::convert::Infallible;
use hyper::{Body, Request, Response, Method, StatusCode, header::AUTHORIZATION};
use futures::future::BoxFuture;
use super::super::database::get_database;
use mongodb::bson::{doc, Document};
use super::super::hash::hash;
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};

pub const METHOD: &Method = &Method::POST;

pub fn get_endpoints() -> Vec<&'static str> {
  vec!["logout", "signout"]
}

pub fn run(req: Request<Body>) -> BoxFuture<'static, Result<Response<Body>, Infallible>> {
  Box::pin(async move {
    let auth = req.headers().get(AUTHORIZATION);

    let db = get_database().await;
    let tokens = db.collection::<Document>("tokens");

    if let Some(token) = auth {
      let mut hashed_token = [0; 32];
      hash::<Sha256>(token.to_str().unwrap(), &std::env::var("TOKEN_SALT").unwrap(), &mut hashed_token);
      let str_hash_token = general_purpose::STANDARD.encode(&hashed_token);

      if let Some(_) = tokens.find_one(doc! { "value": &str_hash_token }, None).await.unwrap() {

        tokens.delete_one(doc! { "value": &str_hash_token }, None).await.unwrap();

        return Ok(Response::new(Body::from("logged out")))
      }

      return Ok(Response::builder().status(StatusCode::UNAUTHORIZED).body(Body::empty()).unwrap())
    }
    
    Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::empty()).unwrap())
  })
}

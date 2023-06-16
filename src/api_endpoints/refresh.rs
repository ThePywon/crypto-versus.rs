use std::convert::Infallible;
use hyper::{Body, Request, Response, Method, StatusCode};
use futures::future::BoxFuture;
use super::super::database::get_database;
use mongodb::bson::{doc, Document, DateTime};
use super::super::hash::hash;
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};
use uuid::Uuid;

pub const METHOD: &Method = &Method::POST;

pub fn get_endpoints() -> Vec<&'static str> {
  vec!["refresh"]
}

pub fn run(req: Request<Body>) -> BoxFuture<'static, Result<Response<Body>, Infallible>> {
  Box::pin(async move {
    let auth = req.headers().get("authorization");

    let db = get_database().await;
    let tokens = db.collection::<Document>("tokens");

    if let Some(token) = auth {
      let mut hashed_token = [0; 32];
      hash::<Sha256>(token.to_str().unwrap(), &std::env::var("TOKEN_SALT").unwrap(), &mut hashed_token);
      let str_hash_token = general_purpose::STANDARD.encode(&hashed_token);

      if let Some(old_token) = tokens.find_one(doc! { "value": &str_hash_token }, None).await.unwrap() {
        if old_token.get_i64("valid_until").unwrap() >= DateTime::now().timestamp_millis() {
          let new_token = Uuid::new_v4().to_string();
          hash::<Sha256>(&new_token, &std::env::var("TOKEN_SALT").unwrap(), &mut hashed_token);
          let new_hash_token = general_purpose::STANDARD.encode(&hashed_token);
          let timestamp = DateTime::now().timestamp_millis() + 300000;

          tokens.update_one(doc! { "value": str_hash_token }, doc! { "$set": { "value": new_hash_token, "valid_until": timestamp } }, None).await.unwrap();

          return Ok(Response::new(Body::from(new_token)))
        }
        else {
          tokens.delete_one(doc! { "value": &str_hash_token }, None).await.unwrap();
        }
      }

      return Ok(Response::builder().status(StatusCode::UNAUTHORIZED).body(Body::empty()).unwrap())
    }

    Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::empty()).unwrap())
  })
}

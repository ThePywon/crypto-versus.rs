use std::convert::Infallible;
use hyper::{Body, Request, Response, Method, StatusCode};
use serde::{Serialize, Deserialize};
use futures::future::BoxFuture;
use super::super::super::database::get_database;
use mongodb::bson::{doc, Document, DateTime};
use super::super::super::hash::hash;
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};

#[derive(Serialize, Deserialize)]
struct StatsData<'a> {
  username: &'a str
}

pub const METHOD: &Method = &Method::POST;

pub fn get_endpoints() -> Vec<&'static str> {
  vec!["/stats", "/statistics"]
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
          let users = db.collection::<Document>("users");
          let user = users.find_one(doc! { "_id": old_token.get_object_id("user_id").unwrap() }, None ).await.unwrap().unwrap();
          let stats = StatsData { username: user.get_str("username").unwrap() };

          return Ok(Response::new(Body::from(serde_json::to_string(&stats).unwrap())))
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
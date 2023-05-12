use std::convert::Infallible;
use hyper::{Body, Request, Response, Method, StatusCode};
use serde::{Serialize, Deserialize};
use futures::future::BoxFuture;
use super::super::database::get_database;
use mongodb::bson::{doc, Document, DateTime};
use super::super::hash::hash;
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct SignupData<'a> {
  username: &'a str,
  password: &'a str
}

pub const METHOD: &Method = &Method::POST;

pub fn get_endpoints() -> Vec<&'static str> {
  vec!["/signup"]
}

pub fn run(req: Request<Body>) -> BoxFuture<'static, Result<Response<Body>, Infallible>> {
  Box::pin(async {
    let body = hyper::body::to_bytes(req.into_body()).await;

    let db = get_database().await;
    let users = db.collection::<Document>("users");
    let tokens = db.collection::<Document>("tokens");

    if let Result::Ok(data) = body {
      if let Result::Ok(data) = serde_json::from_slice::<SignupData>(&data) {
        if let None = users.find_one(doc! { "username": data.username }, None).await.unwrap() {

          let mut hashed_password = [0; 32];
          hash::<Sha256>(data.password, &std::env::var("USER_SALT").unwrap(), &mut hashed_password);
          let str_hash = general_purpose::STANDARD.encode(&hashed_password);

          users.insert_one(doc! { "username": data.username, "password": str_hash }, None).await.unwrap();
          let user_id = users.find_one(doc! { "username": data.username }, None).await.unwrap().unwrap().get_object_id("_id").unwrap();

          let token = Uuid::new_v4().to_string();
          let mut hashed_token = [0; 32];
          hash::<Sha256>(&token, &std::env::var("TOKEN_SALT").unwrap(), &mut hashed_token);
          let str_hash_token = general_purpose::STANDARD.encode(&hashed_token);

          let timestamp = DateTime::now().timestamp_millis() + 300000;
          
          tokens.insert_one(doc! { "user_id": user_id, "value": &str_hash_token, "valid_until": timestamp }, None).await.unwrap();

          Ok(Response::new(Body::from(token)))
        }
        else {
          Ok(Response::builder().status(StatusCode::FORBIDDEN).body(Body::from("User already exists")).unwrap())
        }
      }
      else {
        Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::empty()).unwrap())
      }
    }
    else {
      Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::empty()).unwrap())
    }
  })
}

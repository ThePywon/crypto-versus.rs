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
struct LoginData<'a> {
  username: &'a str,
  password: &'a str
}

pub const METHOD: &Method = &Method::POST;

pub fn get_endpoints() -> Vec<&'static str> {
  vec!["login", "signin"]
}

pub fn run(req: Request<Body>) -> BoxFuture<'static, Result<Response<Body>, Infallible>> {
  Box::pin(async move {
    let auth = req.headers().get("authorization");

    let db = get_database().await;
    let users = db.collection::<Document>("users");
    let tokens = db.collection::<Document>("tokens");

    if let Some(auth_value) = auth {
      let raw_auth = general_purpose::STANDARD.decode(auth_value);

      if let Result::Ok(data) = raw_auth {
        let str_data = std::str::from_utf8(&data).unwrap();

        if str_data.contains(":") {
          let (username, password) = str_data.split_once(":").unwrap();

          let mut hashed_password = [0; 32];
          hash::<Sha256>(password, &std::env::var("USER_SALT").unwrap(), &mut hashed_password);
          let str_hash_pass = general_purpose::STANDARD.encode(&hashed_password);

          if let Some(user) = users.find_one(doc! { "username": username, "password": &str_hash_pass }, None).await.unwrap() {

            let user_id = user.get_object_id("_id").unwrap();
            let old_token = tokens.find_one(doc! { "user_id": user_id }, None).await.unwrap();
            let timestamp = DateTime::now().timestamp_millis() + 300000;
            
            if let None = old_token {
              
              let token = Uuid::new_v4().to_string();
              let mut hashed_token = [0; 32];
              hash::<Sha256>(&token, &std::env::var("TOKEN_SALT").unwrap(), &mut hashed_token);
              let str_hash_token = general_purpose::STANDARD.encode(&hashed_token);

              tokens.insert_one(doc! { "user_id": user_id, "value": &str_hash_token, "valid_until": timestamp }, None).await.unwrap();

              return Ok(Response::new(Body::from(token)))
            }
            else if old_token.unwrap().get_i64("valid_until").unwrap() < DateTime::now().timestamp_millis() {

              let token = Uuid::new_v4().to_string();
              let mut hashed_token = [0; 32];
              hash::<Sha256>(&token, &std::env::var("TOKEN_SALT").unwrap(), &mut hashed_token);
              let str_hash_token = general_purpose::STANDARD.encode(&hashed_token);

              tokens.update_one(doc! { "user_id": user_id }, doc! { "$set": { "value": &str_hash_token, "valid_until": timestamp } }, None).await.unwrap();

              return Ok(Response::new(Body::from(token)))
            }

            return Ok(Response::builder().status(StatusCode::TOO_MANY_REQUESTS).body(Body::empty()).unwrap())
          }
        }

        return Ok(Response::builder().status(StatusCode::UNAUTHORIZED).body(Body::from("Invalid credentials")).unwrap())
      }
    }

    Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::empty()).unwrap())
  })
}

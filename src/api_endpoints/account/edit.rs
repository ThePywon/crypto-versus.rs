use std::convert::Infallible;
use hyper::{Body, Request, Response, Method, StatusCode, header::AUTHORIZATION};
use serde::{Serialize, Deserialize};
use futures::future::BoxFuture;
use super::super::super::database::get_database;
use mongodb::bson::{doc, Document, DateTime};
use super::super::super::hash::hash;
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};

#[derive(Serialize, Deserialize)]
struct EditData<'a> {
  username: Option<&'a str>,
  password: Option<&'a str>
}

pub const METHOD: &Method = &Method::POST;

pub fn get_endpoints() -> Vec<&'static str> {
  vec!["/account/edit"]
}

pub fn run(req: Request<Body>) -> BoxFuture<'static, Result<Response<Body>, Infallible>> {
  Box::pin(async move {
    let (parts, b) = req.into_parts();
    let body = hyper::body::to_bytes(b).await;

    let token = parts.headers.get(AUTHORIZATION).unwrap();

    let db = get_database().await;
    let users = db.collection::<Document>("users");
    let tokens = db.collection::<Document>("tokens");

    let mut hashed_token = [0; 32];
    hash::<Sha256>(token.to_str().unwrap(), &std::env::var("TOKEN_SALT").unwrap(), &mut hashed_token);
    let str_hash_token = general_purpose::STANDARD.encode(&hashed_token);

    if let Some(old_token) = tokens.find_one(doc! { "value": &str_hash_token }, None).await.unwrap() {
      
      if old_token.get_i64("valid_until").unwrap() >= DateTime::now().timestamp_millis() {
        if let Ok(data) = body {
          if let Result::Ok(data) = serde_json::from_slice::<EditData>(&data) {

            let mut str_hash_pass: Option<String> = None;
            if let Some(pass) = data.password {
              let mut hashed_password = [0; 32];
              hash::<Sha256>(pass, &std::env::var("USER_SALT").unwrap(), &mut hashed_password);
              str_hash_pass = Some(general_purpose::STANDARD.encode(&hashed_password));
            }
            
            let mut final_edit: Document;

            if let Some(u) = data.username {
              if let Some(p) = str_hash_pass {
                final_edit = doc! { "username": u, "password": p };
              }
              else {
                final_edit = doc! { "username": u };
              }
            }
            else {
              if let Some(p) = str_hash_pass {
                final_edit = doc! { "password": p };
              }
              else {
                return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::empty()).unwrap())
              }
            }

            users.update_one(doc! { "_id": old_token.get_object_id("user_id").unwrap() }, doc! { "$set": final_edit }, None).await.unwrap();
  
            return Ok(Response::new(Body::from("changes applied")))
          }
        }
  
        return Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::empty()).unwrap())
      }
      else {
        tokens.delete_one(doc! { "value": &str_hash_token }, None).await.unwrap();
      }
    }

    return Ok(Response::builder().status(StatusCode::UNAUTHORIZED).body(Body::empty()).unwrap())
  })
}
/*
pub fn run(req: Request<Body>) -> BoxFuture<'static, Result<Response<Body>, Infallible>> {
  Box::pin(async {
    let body = hyper::body::to_bytes(req.into_body()).await;

    let db = get_database().await;
    let users = db.collection::<Document>("users");
    let tokens = db.collection::<Document>("tokens");

    if let Result::Ok(data) = body {
      if let Result::Ok(data) = serde_json::from_slice::<LoginData>(&data) {

        let mut hashed_password = [0; 32];
        hash::<Sha256>(data.password, &std::env::var("USER_SALT").unwrap(), &mut hashed_password);
        let str_hash_pass = general_purpose::STANDARD.encode(&hashed_password);

        if let Some(user) = users.find_one(doc! { "username": data.username, "password": &str_hash_pass }, None).await.unwrap() {

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

        return Ok(Response::builder().status(StatusCode::UNAUTHORIZED).body(Body::from("Invalid credentials")).unwrap())
      }
    }

    Ok(Response::builder().status(StatusCode::BAD_REQUEST).body(Body::empty()).unwrap())
  })
}
*/
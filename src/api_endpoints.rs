use std::{convert::Infallible, collections::hash_map::HashMap};
use hyper::{Body, Request, Response, Method};
use futures::future::BoxFuture;
mod login;
mod signup;

pub fn get_endpoints() -> HashMap<(&'static Method, &'static str), fn(Request<Body>) -> BoxFuture<'static, Result<Response<Body>, Infallible>>> {
	let mut endpoints = HashMap::new();
	for endpoint in login::get_endpoints() {
		endpoints.insert((login::METHOD, endpoint), login::run as fn(Request<Body>) -> BoxFuture<'static, Result<Response<Body>, Infallible>>);
	}
	for endpoint in signup::get_endpoints() {
		endpoints.insert((signup::METHOD, endpoint), signup::run as fn(Request<Body>) -> BoxFuture<'static, Result<Response<Body>, Infallible>>);
	}
	endpoints
}

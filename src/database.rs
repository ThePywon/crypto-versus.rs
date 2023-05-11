use mongodb::{Client, options::ClientOptions, Database};

pub async fn get_database() -> Database {
  let client_options = ClientOptions::parse(std::env::var("DB_URL").unwrap()).await.unwrap();
  let client = Client::with_options(client_options).expect("Could not connect to database!");

  client.database(&std::env::var("DB_NAME").unwrap())
}
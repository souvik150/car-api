pub struct Config {
  pub mongodb_uri: String,
  pub env: String,
}

impl Config {
  pub fn new() -> Self {
      dotenvy::dotenv().ok();
      let uri = dotenvy::var("MONGODB_URI").unwrap_or_else(|_| "".to_string());
      let env = dotenvy::var("ENV").unwrap_or_else(|_| "dev".to_string());
      Self {
          mongodb_uri: uri,
          env,
      }
  }
}

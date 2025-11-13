use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub jwt_expires_in: i64,
    pub jwt_refresh_expires_in: i64,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env file");

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("SERVER_PORT must be a valid number");

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_secret_key".to_string());

        let jwt_expires_in = env::var("JWT_EXPIRES_IN")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .expect("JWT_EXPIRES_IN must be a valid number");

        let jwt_refresh_expires_in = env::var("JWT_REFRESH_EXPIRES_IN")
            .unwrap_or_else(|_| "2592000".to_string())
            .parse()
            .expect("JWT_REFRESH_EXPIRES_IN must be a valid number");

        Config {
            database_url,
            server_port,
            jwt_secret,
            jwt_expires_in,
            jwt_refresh_expires_in,
        }
    }
}
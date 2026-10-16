#[derive(Clone)]
pub struct AppConfig {
    pub oracle_user: String,
    pub oracle_password: String,
    pub oracle_connect_string: String,
    pub listen_port: u16,
}
impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let listen_port = std::env::var("LISTEN_PORT").expect("LISTEN_PORT not set");
        Self {
            oracle_user: std::env::var("ORACLE_USER").expect("ORACLE_USER not set"),
            oracle_password: std::env::var("ORACLE_PASSWORD").expect("ORACLE_PASSWORD not set"),
            oracle_connect_string: std::env::var("ORACLE_CONNECT_STRING")
                .expect("ORACLE_CONNECT_STRING not set"),
            listen_port: listen_port
                .parse::<u16>()
                .expect("unable to parse LISTEN_PORT"),
        }
    }
}

use axum::Json;
use axum::http::StatusCode;
use oracle::Connection;
use oracle::pool::{GetMode, Pool, PoolBuilder};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

pub fn build_pool(config: &crate::AppConfig) -> Result<Pool, String> {
    let mut builder = PoolBuilder::new(
        &config.oracle_user,
        &config.oracle_password,
        &config.oracle_connect_string,
    );

    builder
        .min_connections(1)
        .max_connections(50)
        .connection_increment(1)
        .get_mode(GetMode::TimedWait(Duration::from_secs(10)))
        .ping_interval(Some(Duration::from_secs(30)))
        .map_err(|err: oracle::Error| err.to_string())?
        .ping_timeout(Duration::from_secs(5))
        .map_err(|err: oracle::Error| err.to_string())?
        .timeout(Duration::from_secs(300))
        .map_err(|err: oracle::Error| err.to_string())?
        .max_lifetime_connection(Duration::from_secs(3600))
        .map_err(|err: oracle::Error| err.to_string())?;

    let pool = builder
        .build()
        .map_err(|err: oracle::Error| err.to_string())?;

    Ok(pool)
}

pub async fn get_connection(
    state: &crate::AppState,
) -> Result<Connection, (StatusCode, Json<serde_json::Value>)> {
    let pool = &state.pool;
    let mut retries = 5;
    let retry_delay = Duration::from_millis(300);

    while retries > 0 {
        match pool.get() {
            Ok(conn) => {
                // Verify that the pooled connection is still alive and responsive
                match conn.ping() {
                    Ok(_) => return Ok(conn),
                    Err(err) => {
                        eprintln!(
                            "Stale database connection detected via ping, discarding and retrying: {:?}",
                            err
                        );
                        // Dropping `conn` causes ODPI-C to clean up and replace the broken session
                    }
                }
            }
            Err(err) => {
                eprintln!("Database Connection Pool Error: {:?}", err);
            }
        }

        retries -= 1;
        if retries > 0 {
            sleep(retry_delay).await;
        }
    }

    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(
            json!({"status": "failure", "data": "Database is currently unavailable. Please try your request again shortly."}),
        ),
    ))
}

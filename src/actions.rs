use crate::{AppState, db::get_connection};
use axum::{Json, extract::State, http::StatusCode};
use oracle::sql_type::ToSql;
use oracle::{Row, sql_type::OracleType};
use serde::Deserialize;
use serde_json::{Map, Value, json};

#[derive(Debug, Deserialize)]
pub struct ActionInput {
    pub query: String,
    #[serde(default)]
    pub params: Vec<Value>,
}

// JSON value -> Oracle bind value
fn json_to_bind(v: &Value) -> Result<Box<dyn ToSql>, String> {
    Ok(match v {
        Value::Null => Box::new(Option::<String>::None),
        // 11g has no SQL BOOLEAN, so use 1/0
        Value::Bool(b) => Box::new(if *b { 1i32 } else { 0i32 }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Box::new(i)
            } else if let Some(f) = n.as_f64() {
                Box::new(f)
            } else {
                return Err(format!("unsupported number: {n}"));
            }
        }
        Value::String(s) => Box::new(s.clone()),
        other => return Err(format!("unsupported param type: {other}")),
    })
}

// OracleType Row -> JSON value
fn row_to_json(cols: &[String], types: &[OracleType], row: &Row) -> oracle::Result<Value> {
    let mut obj = Map::with_capacity(cols.len());

    for (i, (name, ty)) in cols.iter().zip(types).enumerate() {
        let v = match ty {
            // NUMBER(p,0) -> integer (falls back to string if it overflows i64)
            OracleType::Number(p, 0) if *p > 0 => match row.get::<_, Option<i64>>(i) {
                Ok(v) => v.map_or(Value::Null, Value::from),
                Err(_) => row
                    .get::<_, Option<String>>(i)?
                    .map_or(Value::Null, Value::from),
            },
            OracleType::Int64 => row
                .get::<_, Option<i64>>(i)?
                .map_or(Value::Null, Value::from),
            OracleType::UInt64 => row
                .get::<_, Option<u64>>(i)?
                .map_or(Value::Null, Value::from),

            // NUMBER with scale, FLOAT, BINARY_FLOAT/DOUBLE -> f64
            OracleType::Number(..)
            | OracleType::Float(_)
            | OracleType::BinaryFloat
            | OracleType::BinaryDouble => row
                .get::<_, Option<f64>>(i)?
                .map_or(Value::Null, Value::from),

            // VARCHAR2, CHAR, DATE, TIMESTAMP, CLOB, etc. -> string
            _ => row
                .get::<_, Option<String>>(i)?
                .map_or(Value::Null, Value::from),
        };
        obj.insert(name.clone(), v);
    }

    Ok(Value::Object(obj))
}

pub async fn query(
    State(state): State<AppState>,
    Json(input): Json<ActionInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    println!("query: {}", input.query);
    println!("params: {:?}", input.params);

    // Allow Only SELECT statement, others blocked here.
    let first_word = input
        .query
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_uppercase();

    if first_word.as_str() != "SELECT" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"status": "failure", "data": "SELECT statement only allowed"})),
        ));
    }

    // Reject bad params up front with a 400
    for (i, p) in input.params.iter().enumerate() {
        if let Err(e) = json_to_bind(p) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "failure", "data": format!("param {}: {e}", i + 1)})),
            ));
        }
    }

    let conn = get_connection(&state).await?;
    let sql = input.query.clone();
    let params = input.params.clone();

    // The oracle crate is blocking, so keep it off the async runtime threads
    let data = tokio::task::spawn_blocking(move || -> oracle::Result<Vec<Value>> {
        // Build binds inside the closure (Box<dyn ToSql> isn't guaranteed Send)
        let binds: Vec<Box<dyn ToSql>> = params
            .iter()
            .map(|p| json_to_bind(p).expect("validated above"))
            .collect();
        let refs: Vec<&dyn ToSql> = binds.iter().map(|b| b.as_ref()).collect();

        let rs = conn.query(&sql, &refs)?;

        let cols: Vec<String> = rs
            .column_info()
            .iter()
            .map(|c| c.name().to_string())
            .collect();
        let types: Vec<OracleType> = rs
            .column_info()
            .iter()
            .map(|c| c.oracle_type().clone())
            .collect();

        let mut out = Vec::new();
        for row in rs {
            out.push(row_to_json(&cols, &types, &row?)?);
        }
        Ok(out)
    })
    .await
    .map_err(|e| {
        eprintln!("Join error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "failure", "data": "Internal server error"})),
        )
    })?
    .map_err(|err| {
        eprintln!("Database Query Error: {:?}", err);
        match err.db_error().map(|d| d.code()) {
            Some(8002) => (
                StatusCode::BAD_REQUEST,
                Json(
                    json!({"status": "failure", "data": "No current value found in this session"}),
                ),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "failure", "data": "Database operational error occurred"})),
            ),
        }
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "data": data
        })),
    ))
}

pub async fn execute(
    State(state): State<AppState>,
    Json(input): Json<ActionInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    println!("query: {}", input.query);
    println!("params: {:?}", input.params);

    // Only allow DML. DDL (DROP/ALTER/...) is blocked here.
    let first_word = input
        .query
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_uppercase();

    if !matches!(first_word.as_str(), "INSERT" | "UPDATE" | "DELETE") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"status": "failure", "data": "Only INSERT, UPDATE and DELETE statements are allowed"}),
            ),
        ));
    }

    // Reject bad params up front with a 400
    for (i, p) in input.params.iter().enumerate() {
        if let Err(e) = json_to_bind(p) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "failure", "data": format!("param {}: {e}", i + 1)})),
            ));
        }
    }

    let conn = get_connection(&state).await?;
    let sql = input.query.clone();
    let params = input.params.clone();

    let rows_affected = tokio::task::spawn_blocking(move || -> oracle::Result<u64> {
        // Build binds inside the closure (Box<dyn ToSql> isn't guaranteed Send)
        let binds: Vec<Box<dyn ToSql>> = params
            .iter()
            .map(|p| json_to_bind(p).expect("validated above"))
            .collect();
        let refs: Vec<&dyn ToSql> = binds.iter().map(|b| b.as_ref()).collect();

        let stmt = match conn.execute(&sql, &refs) {
            Ok(s) => s,
            Err(e) => {
                let _ = conn.rollback();
                return Err(e);
            }
        };
        let n = stmt.row_count()?;
        conn.commit()?;
        Ok(n)
    })
    .await
    .map_err(|e| {
        eprintln!("Join error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "failure", "data": "Internal server error"})),
        )
    })?
    .map_err(|err| {
        eprintln!("Database Execute Error: {:?}", err);
        match err.db_error().map(|d| d.code()) {
            Some(1) => (
                StatusCode::CONFLICT,
                Json(json!({"status": "failure", "data": "Duplicate record"})),
            ),
            Some(1400) => (
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "failure", "data": "A required column is NULL"})),
            ),
            Some(1861) | Some(1843) | Some(1847) | Some(1858) => (
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "failure", "data": "Invalid date, use YYYY-MM-DD"})),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "failure", "data": "Database operational error occurred"})),
            ),
        }
    })?;

    let data = json!({"status": "success", "data": {"rows_affected": rows_affected}});

    Ok((StatusCode::OK, Json(data)))
}

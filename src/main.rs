#![allow(dead_code)]

use axum::extract::State;
use axum::{Json, Router};
use axum::routing::get;
use serde::Serialize;
use sqlx::{mssql::MssqlPool, prelude::FromRow};
use tokio::net::TcpListener;
use std::env;
use std::sync::Arc;
use dotenv::dotenv;

#[derive(Clone)]
struct AppState {
    mssql: Arc<MssqlPool>,
}

#[derive(Debug, FromRow, Serialize)]
struct Ar {
    code: String,
    name1: String,
}

#[derive(Debug, FromRow, Serialize)]
struct ArInvoice {
    docno: String,
    arcode: String,
    count_row: i32,
}

async fn ar_get(
    State(state): State<AppState>
) -> Json<Vec<Ar>> {
    let rows = sqlx::query_as::<_, Ar>("
        SELECT 
            CAST(Code AS NVARCHAR(100)) AS code,
            CAST(Name1 AS NVARCHAR(200)) AS name1
        FROM BCAR
    ")
    .fetch_all(state.mssql.as_ref())
    .await
    .unwrap();

    println!("AR: {:?}", rows);

    Json(rows)
}

async fn ar_invoice_get(
    State(state): State<AppState>
) -> Json<Vec<ArInvoice>> {
    let rows = sqlx::query_as::<_, ArInvoice>("
        SELECT 
            CAST(DocNo AS NVARCHAR(100)) AS docno,
            CAST(ArCode AS NVARCHAR(200)) AS arcode,
            (SELECT COUNT(*) FROM BCARINVOICE) AS count_row
        FROM BCARINVOICE
    ")
    .fetch_all(state.mssql.as_ref())
    .await
    .unwrap();

    println!("ArInvoice: {:?}", rows);

    Json(rows)
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    let db_url = env::var("DATABASE_URL").unwrap();
    let pool = Arc::new(MssqlPool::connect(&db_url).await?);

    let state = AppState { mssql: pool.clone() };

    let app = Router::new()
        .route("/api/ar", get(ar_get)).with_state(state.clone())
        .route("/api/ar-invoice", get(ar_invoice_get)).with_state(state.clone());

    let addr = "127.0.0.1:3030";
    println!("App running on: {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
   
    Ok(())
}



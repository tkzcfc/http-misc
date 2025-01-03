use actix_web::web;
use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::prelude::*;
use clap::Parser;
use serde::Serialize;
use serde_json::to_string;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

#[derive(Serialize, Debug)]
struct QueryIpResponse {
    status: String,
    query: String,
}

#[get("api/query_ip/json")]
pub async fn api_query_ip_json(req: HttpRequest) -> actix_web::Result<impl Responder> {
    let response = QueryIpResponse {
        status: "success".into(),
        query: if let Some(x) = req.connection_info().realip_remote_addr() {
            x.to_string()
        } else {
            "127.0.0.1".into()
        },
    };

    let body = to_string(&response)?;
    Ok(HttpResponse::Ok().body(body))
}

#[get("adjust_pass_back")]
async fn adjust_pass_back(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    // 获取查询参数
    let query_string = req.query_string();
    // 获取当前时间
    let now: DateTime<Utc> = Utc::now();
    let datetime_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let log_entry = format!("{} $ {}\n", datetime_str, query_string);

    let mut file = data.adjust_pass_back_file.lock().await;
    file.write_all(log_entry.as_bytes()).await?;

    Ok(HttpResponse::Ok().body("ok"))
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The address to listen on
    #[arg(long, default_value = "0.0.0.0:8000")]
    listen_addr: String,
}

// 定义应用状态，用于存储日志文件
struct AppState {
    adjust_pass_back_file: Mutex<tokio::fs::File>,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let adjust_pass_back_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("adjust_pass_back.log")
        .await?;

    let app_state = web::Data::new(AppState {
        adjust_pass_back_file: Mutex::new(adjust_pass_back_file),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(api_query_ip_json)
            .service(adjust_pass_back)
    })
    .bind(&args.listen_addr)?
    .run()
    .await?;

    Ok(())
}

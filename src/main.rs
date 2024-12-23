use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, Responder};
use clap::Parser;
use serde::Serialize;
use serde_json::to_string;

#[derive(Serialize, Debug)]
struct QueryIpResponse {
    status: String,
    query: String,
}

#[get("api/query_ip/json")]
pub async fn api_query_ip_json(req: HttpRequest) -> actix_web::Result<impl Responder> {
    let response = QueryIpResponse {
        status: "status".into(),
        query: if let Some(x) = req.connection_info().realip_remote_addr() {
            x.to_string()
        } else {
            "127.0.0.1".into()
        },
    };

    let body = to_string(&response)?;
    Ok(HttpResponse::Ok().body(body))
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The address to listen on
    #[arg(long, default_value = "0.0.0.0:80")]
    listen_addr: String,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    HttpServer::new(move || App::new().service(api_query_ip_json))
        .bind(&args.listen_addr)?
        .run()
        .await?;

    Ok(())
}

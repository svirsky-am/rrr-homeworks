use blog_server::{run_server, ServerConfig};
use dotenvy::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = ServerConfig::default();
    
    let handle = run_server(config).await?.handle.await;
    handle.unwrap()
}
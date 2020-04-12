mod server;
mod store;
mod utils;

#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();
    server::serve().await;
}

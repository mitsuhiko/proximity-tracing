mod server;
mod utils;
mod store;

#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();
    server::serve().await;
}

use bollard::Docker;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let docker = Docker::connect_with_http_defaults().unwrap();
    let version = docker.version().await.unwrap();
    println!("{:?}", version);
    Ok(())
}

use log::info;
use tonic;

use proto::hello_world::greeter_client::GreeterClient;
use proto::hello_world::HelloRequest;

mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    info!("Server responded: {:?}", response);

    Ok(())
}

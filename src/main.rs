use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, Error, HttpResponse, HttpServer, Responder};
use log::{error, info};
use tonic;

use proto::hello_world::greeter_client::GreeterClient;
use proto::hello_world::HelloRequest;

mod proto;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(data: web::Data<AppState>, req_body: String) -> Result<HttpResponse, Error> {
    let app_state = data.as_ref().clone();

    let mut grpc_client = app_state.grpc_client;
    let request = tonic::Request::new(HelloRequest { name: req_body });

    let grpc_response = match grpc_client.say_hello(request).await {
        Ok(response) => {
            info!("Server responded: {:?}", response);

            response.into_inner().message
        }
        Err(error) => {
            error!("Error: {:?}", error);
            "An error occurred parsing response from gRPC server".to_owned()
        }
    };

    let response_body = format!("{}\n", grpc_response); // We add a newline for readability
    Ok(HttpResponse::Ok().body(response_body))
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[derive(Clone)]
struct AppState {
    grpc_client: GreeterClient<tonic::transport::channel::Channel>,
}

impl AppState {
    fn new(
        grpc_client: GreeterClient<tonic::transport::channel::Channel>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { grpc_client })
    }
}

struct Server {
    thread: Option<std::thread::JoinHandle<()>>,
    system: actix_rt::System,
    http_server: actix_web::dev::Server,
}

impl Server {
    pub fn new(
        host_port: &'static str,
        grpc_client: GreeterClient<tonic::transport::channel::Channel>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let app_state = AppState::new(grpc_client)?;
        let (sender, receiver) = std::sync::mpsc::channel();

        let handle = std::thread::spawn(move || {
            let system = actix_rt::System::new("server");

            let server = HttpServer::new(move || {
                App::new()
                    .data(app_state.clone())
                    .wrap(Logger::default())
                    .service(hello)
                    .service(echo)
                    .route("/hey", web::get().to(manual_hello))
            })
            .bind(host_port);

            let server = match server {
                Ok(s) => s,
                Err(e) => {
                    sender.send(Err(e)).expect("main thread closed channel");
                    return;
                }
            };

            let server = server.run();
            sender
                .send(Ok((actix_rt::System::current(), server)))
                .expect("main thread closed channel");
            let _ = system.run();
        });

        let (system, server) = receiver.recv()??;

        Ok(Server {
            thread: Some(handle),
            system,
            http_server: server,
        })
    }

    pub fn wait(mut self) -> std::thread::Result<()> {
        if let Some(x) = self.thread.take() {
            x.join()
        } else {
            Ok(())
        }
    }

    pub fn shutdown(&self) {
        let graceful = true;
        futures::executor::block_on(self.http_server.stop(graceful));
        self.system.stop();
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.shutdown();
        if let Some(x) = self.thread.take() {
            let _ = x.join();
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grpc_client = GreeterClient::connect("http://[::1]:50051").await?;

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let host_port = "127.0.0.1:8080";
    let server = Server::new(&host_port, grpc_client)?;

    server.wait().expect("Panic on main server thread");

    Ok(())
}

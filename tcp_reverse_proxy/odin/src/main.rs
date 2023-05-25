use tokio::net::{TcpListener, TcpStream};
use tokio::io;
use tokio::select;
use structopt::StructOpt;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use futures::lock::Mutex;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tokio_stream::{StreamExt, StreamMap};
use tokio_stream::wrappers::TcpListenerStream;

//Supprt the config
#[derive(Debug, StructOpt)]
#[structopt(
    name = "odin proxy server",
    about = "Rust proxy with load balancing"
)]
//pass the config file as a command line argument.
struct CmdLine {
    #[structopt(
        short = "f",
        long = "config-file",
        help = "configuration file (accepts YAML, TOML, JSON)"
    )]
    config_file: String,
}
    //used for deserializing the ports in the json
#[derive(Deserialize, Serialize, Debug, Clone)]
struct App {
    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "Ports")]
    ports: Vec<u16>,

    #[serde(rename = "Targets")]
    targets: Vec<String>
}

//Set up load balancing for the proxy. Round Robin has been selected.
pub trait Backend: Sync + Send {
    fn get(&mut self) -> Option<String>;
    fn add(&mut self, backend_str: &str) -> Result<(), Box<dyn Error>>;
    fn remove(&mut self, backend_str: &str) -> Result<(), Box<dyn Error>>;
}
    //set up round-robin for load balancing in the backend
pub struct RoundRobinBackend {
    backends: Vec<String>,
    last_used: usize,
}
    //implement the round robin for the backend
impl RoundRobinBackend {
    pub fn new(backends_str: Vec<String>) -> Result<RoundRobinBackend, Box<dyn Error>> {
        Ok(RoundRobinBackend {
            backends: backends_str,
            last_used: 0,
        })
    }
}

impl Backend for RoundRobinBackend {
    fn get(&mut self) -> Option<String> {
        if self.backends.is_empty() {
            return None;
        }
        self.last_used = (self.last_used + 1) % self.backends.len();
        self.backends.get(self.last_used).map(|b| b.clone())
    }

    fn add(&mut self, backend_str: &str) -> Result<(), Box<dyn Error>> {
        self.backends.push(String::from(backend_str));
        Ok(())
    }

    fn remove(&mut self, backend_str: &str) -> Result<(), Box<dyn Error>> {
        self.backends.retain(|x| x != &String::from(backend_str));
        Ok(())
    }
}


#[derive(Deserialize, Serialize, Debug)]
struct Config {

    #[serde(rename = "Apps")]
    apps: Vec<App>
}

#[tokio::main] //call the tokio macro
async fn main() -> Result<(), Box<dyn Error>> {

    //Accept command line arguments while building
    let args = CmdLine::from_args();
    let config = {
        let config_content = std::fs::read_to_string(&args.config_file)?;
        serde_json::from_str::<Config>(&config_content).unwrap()
    };

    println!("Running proxy on server");

    //Listens on the ports mentioned in the config file and sends the request to the apt target.
    let mut app_proxies = vec![];

    for app in config.apps {
        app_proxies.push(tokio::spawn(async move {
            run_app(app.clone()).await;
        }));
    }
    join_all(app_proxies).await;

    Ok(())
}
 //Run the backend
async fn run_app(app: App) -> Result<(), Box<dyn Error>> {

    let backend: Arc<Mutex<dyn Backend>> = Arc::new(Mutex::new(RoundRobinBackend::new(app.targets).unwrap()));
    let mut client_stream = StreamMap::new();
    for port in app.ports {
        client_stream.insert(port, TcpListenerStream::new(TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).await.unwrap()));
    }

    while let Some((_, conn)) = client_stream.next().await {
        let client_stream = conn?;
        tokio::spawn(handle_connection(client_stream, backend.clone()));
    }

    Ok(())
}
    //configure the proxy to handle incoming connections 
async fn handle_connection(client: TcpStream, backend: Arc<Mutex<dyn Backend>>) {
    if let Ok(server) = TcpStream::connect(backend.lock().await.get().unwrap()).await {
        let (mut eread, mut ewrite) = client.into_split();
        let (mut oread, mut owrite) = server.into_split();

        let e2o = tokio::spawn(async move { io::copy(&mut eread, &mut owrite).await });
        let o2e = tokio::spawn(async move { io::copy(&mut oread, &mut ewrite).await });

        select! {
            _ = e2o => println!("c2s done"),
            _ = o2e => println!("s2c done"),
        }
    } else {
        println!(
            "couldn't connect"
        );
    } 
}
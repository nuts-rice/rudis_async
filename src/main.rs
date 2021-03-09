mod codec;
use crate::codec::RespCodec;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude::*;
//Tokio-codec allows to convert a stream of bytes from the network into a given type, into a parsed messaged termed as Frame
use std::env;
use tokio_codec::Decoder;

mod commands;
use crate::commands::process_client_request;

//A macro for declaring lazily evaluated statics.
//statics that require code to be required at runtime in order to initalized
lazy_static! {
    static_ref RUDIS_DB: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

fn main() -> Result<(), Box<std::error::Error>> {
    let addr = env::args()
        .skip(1)
        .next()
        .unwrap_or("127.0.0.1:6378".to_owned());
    let addr = addr.parse::<SocketAddr>()?;

    let listener = TcpListener::bind(&addr)?;
    println!("rudis_async listining on: {}", addr);

    //Incoming() method on listener, returns an iterator of new client connections
    let server_future = listener
        .incoming()
        .map_err(|e| println!("Failed to accept socker; error = {:?}", e))
        .for_each(handle_flient);

    //Creates a main tokio task and schedules future for execution
    tokio::run(server_future);
    Ok(())
}

fn handle_client(client: TcpStream) -> Result<(), ()> {
    //Convert stream to a framed future, splits into transmit and reciever
    //Recieves client connection. Converts the framed into a Stream and Sink.
    let (tx, rx) = RespCodec.framed(client).split();
    //Passing process client response, which will resolve future to a decoded frame
    let reply = rx.and_then(process_client_request);
    let task = tx.send_all(reply).then(|res| {
        if let Err(e) = res {
            eprintln!("Failed to process connection; error = {:?}", e);
        }
        Ok(())
    });
    tokio::spawn(task);
    Ok(())
}

extern crate actix;
extern crate futures;
extern crate tokio;

use actix::prelude::*;
use actix::{Actor, Context};

use futures::Stream;

use std::net;
use std::str::FromStr;

use tokio::codec::FramedRead;
use tokio::io::AsyncRead;
use tokio::net::{TcpListener, TcpStream};

use crate::codec::P2PCodec;
use crate::session::Session;

/// Define tcp server that will accept incoming tcp connection and create
/// session actors.
pub struct Server {
    pub port: u16,
}

impl Server {
    /// Method to create a new server
    pub fn new(port: u16) -> Self {
        Server { port }
    }
}

#[derive(Message, Debug)]
/// Struct to hold a tcp stream and its socket addr
struct TcpConnect(pub TcpStream, pub net::SocketAddr);

/// Server handler for TcpConnect messages (built from incoming connections)
impl Handler<TcpConnect> for Server {
    /// this is response for message, which is defined by `ResponseType` trait
    /// in this case we just return unit.
    type Result = ();

    fn handle(&mut self, msg: TcpConnect, _ctx: &mut Self::Context) {
        // Create a session actor
        Session::create(move |ctx| {
            println!("Trying to create server session");

            // Split tcp stream into read and write parts
            let (r, w) = msg.0.split();

            // Add message stream in session from the read part of the tcp stream (with the
            // P2P codec)
            Session::add_stream(FramedRead::new(r, P2PCodec), ctx);

            // Create the session actor and store in it the write part of the tcp stream (with the
            // P2P codec)
            Session::new(actix::io::FramedWrite::new(w, P2PCodec, ctx))
        });
    }
}

/// Make actor from `Server`
impl Actor for Server {
    /// Every actor has to provide execution `Context` in which it can run.
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Create address for the TCP server
        let addr = format!("0.0.0.0:{}", self.port);
        let addr = net::SocketAddr::from_str(&addr).unwrap();

        // Bind TCP listener to this address
        let listener = TcpListener::bind(&addr).unwrap();

        // Add message stream in server which will return a TcpConnect for each incoming TCP
        // connection
        ctx.add_message_stream(listener.incoming().map_err(|_| ()).map(|stream| {
            // Get peer address from the stream
            let addr = stream.peer_addr().unwrap();

            // Return a TcpConnect struct
            TcpConnect(stream, addr)
        }));

        println!("P2P server has been started at {:?}", addr);
    }
}

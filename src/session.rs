use std::{sync::Arc, collections::HashMap, net::SocketAddr};

use tokio::{sync::{mpsc, Mutex}, io, net::TcpStream};

/// Shorthand for the transmit half of the message channel.
type Tx = mpsc::UnboundedSender<String>;

/// Shorthand for the receive half of the message channel.
type Rx = mpsc::UnboundedReceiver<String>;
/// The state for each connected client.
pub struct Shared {
    pub peers: HashMap<SocketAddr, Tx>,
}

impl Shared {
    /// Create a new, empty, instance of `Shared`.
    pub fn new() -> Self {
        Shared {
            peers: HashMap::new(),
        }
    }
}
pub struct Peer {
    /// The TCP socket wrapped with the `Lines` codec, defined below.
    ///
    /// This handles sending and receiving data on the socket. When using
    /// `Lines`, we can work at the line level instead of having to manage the
    /// raw byte operations.
    // lines: Framed<TcpStream, LinesCodec>,

    /// Receive half of the message channel.
    ///
    /// This is used to receive messages from peers. When a message is received
    /// off of this `Rx`, it will be written to the socket.
    pub rx: Rx,
}

impl Peer {
    /// Create a new instance of `Peer`.
    pub async fn new(
        state: Arc<Mutex<Shared>>,
        addr: SocketAddr,
        // lines: Framed<TcpStream, LinesCodec>,
    ) -> io::Result<Peer> {
        // Get the client socket address
        // let addr = stream.peer_addr();
        // let addr = match addr {
        //     Ok(v) => v,
        //     Err(e) => {
        //         return Err(e);
        //     }
        // };

        // Create a channel for this peer
        let (tx, rx) = mpsc::unbounded_channel();

        // Add an entry for this `Peer` in the shared state map.
        state.lock().await.peers.insert(addr, tx);

        Ok(Peer {rx})
        // Ok(Peer { lines, rx })
    }
}
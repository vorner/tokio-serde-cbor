use std::net::{TcpListener as StdTcpListener, TcpStream as StdTcpStream};

use tokio::codec::Decoder;
use tokio::net::tcp::TcpStream;
use tokio::prelude::*;
use tokio::reactor::Handle;
use tokio::runtime::Runtime;

use tokio_serde_cbor::Codec;

use std::collections::HashMap;

type Error = Box<dyn std::error::Error + Send + Sync>;

// We create some test data to serialize. This works because Serde implements
// Serialize and Deserialize for HashMap, so the codec can frame this type.
type TestData = HashMap<String, usize>;

/// Something to test with. It doesn't really matter what it is.
fn test_data() -> TestData {
    let mut data = HashMap::new();
    data.insert("hello".to_owned(), 42);
    data.insert("world".to_owned(), 0);
    data
}

/// Creates a connected pair of sockets.
///
/// This is similar to UnixStream::socket_pair, but works on windows too.
///
/// This is blocking, so it arguably doesn't belong into an async application, but this is not the
/// point of the example here.
fn socket_pair() -> Result<(TcpStream, TcpStream), Error> {
    // port 0 = let the OS choose
    let listener = StdTcpListener::bind("127.0.0.1:0")?;
    let stream1 = StdTcpStream::connect(listener.local_addr()?)?;
    let stream2 = listener.accept()?.0;
    let stream1 = TcpStream::from_std(stream1, &Handle::default())?;
    let stream2 = TcpStream::from_std(stream2, &Handle::default())?;
    Ok((stream1, stream2))
}

fn main() -> Result<(), Error> {
    // This creates a pair of TCP domain sockets that are connected together.
    let (sender_socket, receiver_socket) = socket_pair()?;

    // Create the codec, type annotations are needed here.
    let codec: Codec<TestData, TestData> = Codec::new();

    // Get read and write parts of our streams (we ignore the other directions, but we could
    // .split() them if we wanted to talk both ways).
    let sender = codec.clone().framed(sender_socket);
    let receiver = codec.framed(receiver_socket);

    // This is the data we will send over.
    let msg1 = test_data();
    let msg2 = test_data();

    // Send method comes from Sink and it will return a future we can spawn with tokio.
    // It consumes self, so we need to chain the next send with then.
    let send_all = sender
            .send(msg1)
            .and_then(|sender| sender.send(msg2))
            // Close the sink (otherwise it would get returned throughout the join and block_on and
            // the for_each would wait for more messages).
            .map(|sender| drop(sender));

    // for each frame, thus for each entire object we receive.
    let recv_all = receiver.for_each(|msg| {
        println!("Received: {:#?}", msg);
        Ok(())
    });

    Runtime::new()?.block_on_all(send_all.join(recv_all))?;

    Ok(())
}

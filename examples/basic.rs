use std::collections::HashMap;

use futures::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Decoder;

use tokio_serde_cbor::Codec;

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
async fn socket_pair() -> Result<(TcpStream, TcpStream), Error> {
    // port 0 = let the OS choose
    let mut listener = TcpListener::bind("127.0.0.1:0").await?;
    let stream1 = TcpStream::connect(listener.local_addr()?).await?;
    let stream2 = listener.accept().await?.0;

    Ok((stream1, stream2))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // This creates a pair of TCP domain sockets that are connected together.
    let (sender_socket, receiver_socket) = socket_pair().await?;

    // Create the codec, type annotations are needed here.
    let codec: Codec<TestData, TestData> = Codec::new();

    // Get read and write parts of our streams (we ignore the other directions, but we could
    // .split() them if we wanted to talk both ways).
    let mut sender = codec.clone().framed(sender_socket);
    let receiver = codec.framed(receiver_socket);

    // This is the data we will send over.
    let msg1 = test_data();
    let msg2 = test_data();

    let mut msgs = futures::stream::iter(vec![Ok(msg1), Ok(msg2)]);

    // Send method comes from Sink and it will return a future we can spawn with tokio.
    sender.send_all(&mut msgs).await?;

    // Close the sink (otherwise it would get returned throughout the join and block_on and
    // the for_each would wait for more messages).
    sender.close().await?;

    // for each frame, thus for each entire object we receive.
    let recv_all = receiver.for_each(|msg| {
        println!("Received: {:#?}", msg);
        return future::ready(());
    });

    recv_all.await;

    Ok(())
}

use std::collections::HashMap;

use futures::join;
use futures::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Decoder;

use tokio_serde_cbor::{Codec, Error as CodecError};

type AppError = Box<dyn std::error::Error + Send + Sync>;

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
async fn socket_pair() -> Result<(TcpStream, TcpStream), AppError> {
    // port 0 = let the OS choose
    let mut listener = TcpListener::bind("127.0.0.1:0").await?;
    let stream1 = TcpStream::connect(listener.local_addr()?).await?;
    let stream2 = listener.accept().await?.0;

    Ok((stream1, stream2))
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
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

    // mapping to std::result::Result is needed because SinkExt::send_all
    // expects TryStream - a stream whose item is std::result::Result
    let mut msgs = futures::stream::iter(vec![msg1, msg2].into_iter().map(Ok));

    // Send method comes from Sink and it will return a future we can spawn with tokio.
    let send_all = async {
        sender.send_all(&mut msgs).await?;
        // Close the sink (otherwise it would get returned throughout the join and block_on and
        // the for_each would wait for more messages).
        sender.close().await?;
        Ok::<(), CodecError>(())
    };

    // for each frame, thus for each entire object we receive.
    let recv_all = receiver.for_each(|msg| {
        println!("Received: {:#?}", msg);
        return future::ready(());
    });

    // temporary variable here is type hint for Rust
    // join! returns tuple of all values returned by each joined futures
    // (std::result::Result and () in this case)
    let r: Result<(), AppError> = match join!(send_all, recv_all) {
        (Err(e), _) => Err(Box::new(e)),
        _ => Ok(()),
    };

    r
}

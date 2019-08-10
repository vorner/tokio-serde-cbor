//! Example demonstration how to use the codec with futures 0.3 networking and the futures_codec crate.
//! Run with `cargo run --example futures --feature futures_codec`.

#![feature(async_await)]

type Error = Box<dyn std::error::Error + Send + Sync>;

#[cfg(feature="futures_codec")]
#[runtime::main]
async fn main() -> Result<(), Error> {
    use {runtime::net::{TcpListener, TcpStream}};
    use futures::{SinkExt, StreamExt};
    use futures_codec::Framed;
    use tokio_serde_cbor::Codec;
    use std::collections::HashMap;

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
    async fn socket_pair() -> Result<(TcpStream, TcpStream), Error> {
        // port 0 = let the OS choose
        let mut listener = TcpListener::bind("127.0.0.1:0")?;
        let stream1 = TcpStream::connect(listener.local_addr()?);
        let stream2 = listener.accept();
        Ok((stream1.await?, stream2.await?.0))
    }

    // This creates a pair of TCP domain sockets that are connected together.
    let (sender_socket, receiver_socket) = socket_pair().await?;

    // This is the data we will send over.
    let msg1 = test_data();
    let msg2 = test_data();

    // a task for the sender
    let send_task = async move {
        let mut sender = Framed::new( sender_socket, Codec::<TestData, TestData>::new() );
        sender.send(msg1).await?;
        sender.send(msg2).await?;

        // unfortunately we have to annotate the type here if we want to
        // use the '?' operator inside the async block.
        let res: Result<(), Error> = Ok(());
        res
    };

    // a separate task for the receiver. If would like this to run longer, you would spawn it,
    // but here we send first, then receive, so we just await to keep it simple.
    let receive_task = async move {
        let mut receiver = Framed::new( receiver_socket, Codec::<TestData, TestData>::new() );
        while let Some(msg) = receiver.next().await.transpose()? {
            println!("Received: {:#?}", msg);
        }
        // the compiler can infer the result type here since we return it from main() which has annotated types.
        Ok(())
    };

    send_task.await?;
    receive_task.await
}


#[cfg(not(feature="futures_codec"))]
fn main() -> Result<(), Error>
{
    Err( "Please run this example with --feature futures_codec".into() )
}

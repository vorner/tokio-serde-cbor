#![deny(warnings)]

extern crate tokio_uds;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_io;
extern crate serde;

extern crate tokio_serde_cbor;


use tokio::prelude::*;
use tokio_codec::Decoder;
use tokio_uds::UnixStream;

use tokio_serde_cbor::Codec;

use std::collections::HashMap;


// We create some test data to serialize. This works because Serde implements
// Serialize and Deserialize for HashMap, so the codec can frame this type.
//
type TestData = HashMap<String, usize>;

/// Something to test with. It doesn't really matter what it is.
fn test_data() -> TestData {
	let mut data = HashMap::new();
	data.insert("hello".to_owned(), 42usize);
	data.insert("world".to_owned(), 0usize);
	data
}


fn main() -> Result<(), Box<std::error::Error>>
{
	// This creates a pair of unix domain sockets that are connected together.
	//
	let (socket_a, socket_b) = UnixStream::pair().expect( "Could not create pair of sockets" );

	// Create the codec, type annotations are needed here.
	//
	let codec: Codec<TestData, TestData> = Codec::new();

	// Get read and write parts of our streams
	//
	let (sink_a, _stream_a) = codec.clone().framed( socket_a ).split();
	let (_sink_b, stream_b) = codec        .framed( socket_b ).split();

	// This is the data we will send over.
	//
	let data = test_data();
	let data2 = test_data();

	// Send method comes from tokio::AsyncWrite it will return a future we can spawn with tokio.
	// It consumes self, so we need to chain the next send with then. It returns a result,
	// so normally you would have to properly map the error, but here we just bail with expect.
	//
	let af =

		sink_a.send( data )
		.then( |sink| sink.expect( "First send failed" ).send( data2 ) );

	// for each frame, thus for each entire object we receive.
	//
	let bf = stream_b.for_each( |msg|
	{
		println!( "Received: {:#?}", msg );

		Ok(())

	});


	// Spawn the sender of pongs and then wait for our pinger to finish.
	tokio::run(
	{
		bf.join( af )//.join( af2 )
		  .map_err(|e| println!("error = {:?}", e))
		  .map(|_| {std::process::exit(0);})
	});

	Ok(())
}

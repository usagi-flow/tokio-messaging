extern crate tokio_messaging;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::time::timeout;

use tokio_messaging::*;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum TestMessage
{
	Greeting,
	Request
}

impl Message for TestMessage {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TestData(&'static str);

impl MessageData for TestData {}

fn messaging() -> &'static mut Messaging<TestMessage, TestData>
{
	static mut INSTANCE: Option<Messaging<TestMessage, TestData>> = None;

	unsafe {
		if let None = INSTANCE {
			INSTANCE = Some(Messaging::new());
		}

		return INSTANCE.as_mut().unwrap();
	}
}

/// A single sender sends a message while a single receiver is listening for that message.
/// Ensure the receiver receives the message.
#[tokio::test]
async fn spsc()
{
	let sender_handle = tokio::spawn(async move {
		// Wait a bit until the receiver has started listening
		tokio::time::sleep(Duration::from_millis(500)).await;

		messaging().dispatch(TestMessage::Request, TestData("Hello from sender"));
	});

	let receiver_handle = tokio::spawn(async move {
		let result: Arc<Mutex<Option<TestData>>> = Arc::new(Mutex::new(None));
		let result_to_be_filled = result.clone();
		let task = messaging().on(TestMessage::Request, move |data: TestData| {
			result_to_be_filled.lock().unwrap().replace(data);
		});

		// Wait a bit until the sender has sent and the callback has processed the information, then
		// interrupt the listening after 1s
		timeout(Duration::from_millis(1000), task)
			.await
			.expect_err("The listening task is expected to time out.");

		return result.lock().unwrap().clone();
	});

	let (_sender_result, receiver_result) = tokio::join!(sender_handle, receiver_handle);
	let receiver_result = receiver_result.unwrap();

	assert_eq!(receiver_result.unwrap().0, "Hello from sender");
}

/// A single sender sends a message while a multiple receivers are listening for that message.
/// Ensure each receiver receives the message.
#[tokio::test]
async fn spmc()
{
	let sender_handle = tokio::spawn(async move {
		// Wait a bit until the receiver has started listening
		tokio::time::sleep(Duration::from_millis(500)).await;

		messaging().dispatch(TestMessage::Request, TestData("Hello from sender"));
	});

	let receiver = || async {
		let result: Arc<Mutex<Option<TestData>>> = Arc::new(Mutex::new(None));
		let result_to_be_filled = result.clone();
		let task = messaging().on(TestMessage::Request, move |data: TestData| {
			result_to_be_filled.lock().unwrap().replace(data);
		});

		// Wait a bit until the sender has sent and the callback has processed the information, then
		// interrupt the listening after 1s
		timeout(Duration::from_millis(1000), task)
			.await
			.expect_err("The listening task is expected to time out.");

		return result.lock().unwrap().clone();
	};
	let receiver_handle_a = tokio::spawn(receiver());
	let receiver_handle_b = tokio::spawn(receiver());
	let receiver_handle_c = tokio::spawn(receiver());

	let (_, receiver_a_result, receiver_b_result, receiver_c_result) =
		tokio::join!(sender_handle, receiver_handle_a, receiver_handle_b, receiver_handle_c);
	let receiver_a_result = receiver_a_result.unwrap();
	let receiver_b_result = receiver_b_result.unwrap();
	let receiver_c_result = receiver_c_result.unwrap();

	assert_eq!(receiver_a_result.unwrap().0, "Hello from sender");
	assert_eq!(receiver_b_result.unwrap().0, "Hello from sender");
	assert_eq!(receiver_c_result.unwrap().0, "Hello from sender");
}

/// Multiple senders send a message while a single receiver is listening for that message.
/// Ensure the receiver receives all the messages.
#[tokio::test]
async fn mpsc()
{
	let sender = || async {
		// Wait a bit until the receiver has started listening
		tokio::time::sleep(Duration::from_millis(500)).await;

		messaging().dispatch(TestMessage::Request, TestData("Hello from sender"));
	};
	let sender_handle_a = tokio::spawn(sender());
	let sender_handle_b = tokio::spawn(sender());
	let sender_handle_c = tokio::spawn(sender());

	let receiver_handle = tokio::spawn(async move {
		let result: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
		let result_to_be_filled = result.clone();
		let task = messaging().on(TestMessage::Request, move |_data: TestData| {
			*result_to_be_filled.lock().unwrap() += 1;
		});

		// Wait a bit until the sender has sent and the callback has processed the information, then
		// interrupt the listening after 1s
		timeout(Duration::from_millis(1000), task)
			.await
			.expect_err("The listening task is expected to time out.");

		return result.lock().unwrap().clone();
	});

	let (_, _, _, receiver_result) = tokio::join!(sender_handle_a, sender_handle_b, sender_handle_c, receiver_handle);
	let received_messages = receiver_result.unwrap();

	assert_eq!(received_messages, 3);
}

/// Send a message and listen for another one. Ensure the listener does not receive the sent message.
#[tokio::test]
async fn message_isolation()
{
	let sender_handle = tokio::spawn(async move {
		// Wait a bit until the receiver has started listening
		tokio::time::sleep(Duration::from_millis(500)).await;

		messaging().dispatch(TestMessage::Request, TestData("Hello from sender"));
	});

	let receiver_handle = tokio::spawn(async move {
		let task = messaging().on(TestMessage::Greeting, move |_data: TestData| {
			panic!("Received data; this shouldn't happen.");
		});

		// Wait a bit until the sender has sent and the callback has processed the information, then
		// interrupt the listening after 1s
		timeout(Duration::from_millis(1000), task)
			.await
			.expect_err("The listening task is expected to time out.");
	});

	let results = tokio::join!(sender_handle, receiver_handle);
	results.0.unwrap();
	results.1.unwrap();
}

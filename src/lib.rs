/*!
A crate which offers non-blocking publish/subscribe functionality using Tokio channels.

Publishing messages and subscribing to them is done using an `Messaging` instance,
which acts as a message broker.

In order to create a message broker, start by defining the structure of messages and their data (payload).
The types should implement `Message` and `MessageData` respectively.

```
# use tokio_messaging::*;
#
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum MyMessage
{
	Greeting,
	Request
}

impl Message for MyMessage {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MyPayload(&'static str);

impl MessageData for MyPayload {}
```

Next, create the message broker instance. Usually, you'll have a single, long-living instance.
```
# use lazy_static::lazy_static;
# use tokio_messaging::*;
#
# #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
# enum MyMessage
# {
# 	Greeting,
# 	Request
# }
#
# impl Message for MyMessage {}
#
# #[derive(Clone, Copy, Debug, PartialEq, Eq)]
# struct MyPayload(&'static str);
#
# impl MessageData for MyPayload {}
#
lazy_static! {
	static ref INSTANCE: Messaging<MyMessage, MyPayload> = { Messaging::new() };
}

pub fn messaging() -> &'static Messaging<MyMessage, MyPayload> { &INSTANCE }
```

Publish messages using the `dispatch()` function and subscribe to them using the `on()` function.
```
# use lazy_static::lazy_static;
# use tokio_messaging::*;
#
# #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
# enum MyMessage
# {
# 	Greeting,
# 	Request
# }
#
# impl Message for MyMessage {}
#
# #[derive(Clone, Copy, Debug, PartialEq, Eq)]
# struct MyPayload(&'static str);
#
# impl MessageData for MyPayload {}
#
# lazy_static! {
# 	static ref INSTANCE: Messaging<MyMessage, MyPayload> = { Messaging::new() };
# }
#
# pub fn messaging() -> &'static Messaging<MyMessage, MyPayload> { &INSTANCE }
#
# let mut rt = tokio::runtime::Runtime::new().unwrap();
# rt.block_on(async {
#
# use std::sync::Arc;
# use std::sync::Mutex;
# use std::time::Duration;
#
// Subscribe to messages
tokio::spawn(messaging().on(MyMessage::Request, |data: MyPayload| {
	assert_eq!(data.0, "Here's a request!");
}));

// Publish a message
messaging().dispatch(MyMessage::Request, MyPayload("Here's a request!"));
#
# });
```
*/
mod message_data;
pub use message_data::*;

mod message;
pub use message::*;

mod messaging;
pub use messaging::*;

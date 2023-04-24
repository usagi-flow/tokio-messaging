use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, RwLock};

use tokio::sync::broadcast;

use crate::{Message, MessageData};

/// The message broker which allows publishing and subscribing to messages.
pub struct Messaging<M: Message, D: MessageData>
{
	channels: RwLock<HashMap<M, Arc<(broadcast::Sender<D>, broadcast::Receiver<D>)>>>
}

impl<M: Message, D: MessageData + 'static> Messaging<M, D>
{
	/// Create a new instance of the messaging broker.
	pub fn new() -> Self
	{
		return Messaging {
			channels: RwLock::new(HashMap::new())
		};
	}

	fn get_channel(&self, message: M) -> Arc<(broadcast::Sender<D>, broadcast::Receiver<D>)>
	{
		let mut channels = self.channels.write().unwrap();

		let channel = channels.entry(message).or_insert_with(|| {
			return Arc::new(broadcast::channel::<D>(16));
		});

		return channel.clone();
	}

	/// Dispatch a message to all listeners.
	///
	/// The data may get cloned in the process.
	pub fn dispatch(&self, message: M, data: D)
	{
		let tx = &self.get_channel(message).0;
		tx.send(data.clone()).unwrap();
	}

	/// Listen for a message from any source and invokes the specified callback for each recieved
	/// message.
	pub async fn on<F>(&self, message: M, mut callback: F)
	where
		F: FnMut(D) -> () + Send + 'static
	{
		let tx = &self.get_channel(message).0;
		let mut receiver = tx.subscribe();

		loop {
			if let Ok(data) = receiver.recv().await {
				callback(data.clone());
			}
			else {
				// TODO: a better handling (at least a warning) might be necessary.
				// Err can happen if the receiver lagged behind.
				break;
			}
		}
	}

	/// Listen for a message from any source and invokes the specified callback for each recieved
	/// message.
	pub async fn on_async<F, R>(&self, message: M, mut callback: F)
	where
		F: FnMut(D) -> R + Send + 'static,
		R: Future<Output = ()>
	{
		let mut receiver = self.get_channel(message).0.subscribe();

		loop {
			if let Ok(data) = receiver.recv().await {
				//println!("Message received: {:?}", message);
				callback(data).await;
			}
			else {
				// TODO: a better handling (at least a warning) might be necessary.
				// Err can happen if the receiver lagged behind.
				break;
			}
		}
	}
}

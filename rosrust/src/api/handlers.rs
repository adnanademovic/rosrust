use crate::Message;
use std::collections::HashMap;
use std::marker::PhantomData;

/// Handles all calls involved with a subscription
///
/// Each subscription is done in one thread, so there is no synchronization necessary between
/// these calls.
pub trait SubscriptionHandler<T>: Send + 'static {
    /// Called before any message is accepted from a certain caller ID
    ///
    /// Contains the headers for handling the specific connection
    fn connection(&mut self, headers: HashMap<String, String>);

    /// Called upon receiving any message
    fn message(&mut self, message: T, callerid: &str);
}

pub struct CallbackSubscriptionHandler<T, F, G> {
    on_message: F,
    on_connect: G,
    _phantom: PhantomData<T>,
}

impl<T, F, G> CallbackSubscriptionHandler<T, F, G>
where
    T: Message,
    F: Fn(T, &str) + Send + 'static,
    G: Fn(HashMap<String, String>) + Send + 'static,
{
    pub fn new(on_message: F, on_connect: G) -> Self {
        Self {
            on_message,
            on_connect,
            _phantom: PhantomData,
        }
    }
}

impl<T, F, G> SubscriptionHandler<T> for CallbackSubscriptionHandler<T, F, G>
where
    T: Message,
    F: Fn(T, &str) + Send + 'static,
    G: Fn(HashMap<String, String>) + Send + 'static,
{
    fn connection(&mut self, headers: HashMap<String, String>) {
        (self.on_connect)(headers)
    }

    fn message(&mut self, message: T, callerid: &str) {
        (self.on_message)(message, callerid)
    }
}

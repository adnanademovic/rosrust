use crate::action_client::CommStateMachine;
use crate::Action;
use std::sync::{Arc, Mutex};

pub struct ClientGoalHandle<T: Action> {
    state_machine: Arc<Mutex<CommStateMachine<T>>>,
}

impl<T: Action> ClientGoalHandle<T> {
    pub fn new(state_machine: Arc<Mutex<CommStateMachine<T>>>) -> Self {
        Self { state_machine }
    }
}

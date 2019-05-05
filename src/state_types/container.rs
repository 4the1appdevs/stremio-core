use super::actions::Msg;
use serde::Serialize;
use std::cell::RefCell;
use std::ops::Deref;

pub trait ContainerInterface {
    fn dispatch(&self, action: &Msg) -> bool;
    fn get_state_serialized(&self) -> Result<String, serde_json::Error>;
}

pub trait Container {
    fn dispatch(&self, action: &Msg) -> Option<Box<Self>>;
}

pub struct ContainerHolder<S: Container + 'static>(RefCell<S>);

impl<S> ContainerHolder<S>
where
    S: Container + Clone,
{
    pub fn new(container: S) -> Self {
        ContainerHolder(RefCell::new(container))
    }
    pub fn get_state_owned(&self) -> S {
        self.0.borrow().to_owned()
    }
}

impl<S> ContainerInterface for ContainerHolder<S>
where
    S: Serialize + Container,
{
    fn dispatch(&self, action: &Msg) -> bool {
        let maybe_new_state = self.0.borrow().dispatch(action);
        match maybe_new_state {
            Some(state) => {
                *self.0.borrow_mut() = *state;
                true
            }
            None => false,
        }
    }
    fn get_state_serialized(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self.0.borrow().deref())
    }
}

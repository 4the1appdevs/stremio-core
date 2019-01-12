use super::actions::Action;

// Build a chain of Handlers
// the flow is: .dispatch() -> <all handlers> -> recv
// Each handler can further emit to all of it's following handlers
// This is achieved by building a recursive chain of closures, each one invoking a handler, while using the previous closure (h_taken)
// to pass the action along the chain and give the handler an ability to emit new actions from that point of the chain onward
pub type DispatcherFn = Box<Fn(&Action)>;
pub struct Chain {
    dispatcher: DispatcherFn,
}
impl Chain {
    pub fn new(handlers: Vec<&'static impl Handler>, recv: &'static Fn(&Action)) -> Chain {
        // perhaps this might be helpful to remove the unwraps: https://www.reddit.com/r/rust/comments/64f9c8/idea_replace_with_is_it_safe/
        let mut dispatcher: Option<DispatcherFn> = Some(Box::new(move |action| recv(&action)));
        for handler in handlers.iter().rev() {
            let h_taken = *handler;
            let d_taken = dispatcher.take().unwrap();
            dispatcher = Some(Box::new(move |action| {
                h_taken.handle(&action, &d_taken);
                d_taken(&action);
            }));
        }
        Chain {
            dispatcher: dispatcher.unwrap(),
        }
    }
    pub fn dispatch(&self, action: &Action) {
        (self.dispatcher)(action);
    }
}

pub trait Handler {
    fn handle(&self, action: &Action, emit: &DispatcherFn);
}

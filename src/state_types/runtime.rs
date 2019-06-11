use enclose::*;
use derivative::*;
use std::marker::PhantomData;
use futures::future;
use futures::sync::mpsc::{channel, Sender, Receiver};
use crate::state_types::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub enum RuntimeEv { NewModel, Event(Event) }

#[derive(Derivative)]
#[derivative(Debug, Clone(bound=""))]
pub struct Runtime<Env: Environment, M: Update> {
    pub app: Rc<RefCell<M>>,
    tx: Sender<RuntimeEv>,
    env: PhantomData<Env>
}
impl<Env: Environment + 'static, M: Update + 'static> Runtime<Env, M> {
    pub fn new(app: M, len: usize) -> (Self, Receiver<RuntimeEv>) {
        let (tx, rx) = channel(len);
        let app = Rc::new(RefCell::new(app));
        (Runtime { app, tx, env: PhantomData }, rx)
    }
    pub fn dispatch(&self, msg: &Msg) -> Box<dyn Future<Item = (), Error = ()>> {
        let handle = self.clone();
        let fx = self.app.borrow_mut().update(msg);
        // Send events
        {
            let mut tx = self.tx.clone();
            if fx.has_changed {
                let _ = tx.try_send(RuntimeEv::NewModel);
            }
            if let Msg::Event(ev) = msg {
                let _ = tx.try_send(RuntimeEv::Event(ev.to_owned()));
            }
        }
        // Handle next effects
        let all = fx
            .effects
            .into_iter()
            .map(enclose!((handle) move |ft| ft
                .then(enclose!((handle) move |r| {
                    let msg = match r {
                        Ok(msg) => msg,
                        Err(msg) => msg,
                    };
                    Env::exec(handle.dispatch(&msg));
                    future::ok(())
                }))
            ));
        Box::new(futures::future::join_all(all)
            .map(|_| ()))
    }
}

use crate::runtime::msg::Msg;
use derive_more::From;
use futures::future::LocalBoxFuture;

pub type EffectFuture = LocalBoxFuture<'static, Msg>;

#[derive(From)]
pub enum Effect {
    Msg(Msg),
    Future(EffectFuture),
}

pub struct Effects {
    effects: Vec<Effect>,
    pub has_changed: bool,
}

impl IntoIterator for Effects {
    type Item = Effect;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.effects.into_iter()
    }
}

impl Effects {
    pub fn none() -> Self {
        Effects {
            effects: vec![],
            has_changed: true,
        }
    }
    pub fn one(effect: Effect) -> Self {
        Effects {
            effects: vec![effect],
            has_changed: true,
        }
    }
    pub fn many(effects: Vec<Effect>) -> Self {
        Effects {
            effects,
            has_changed: true,
        }
    }
    pub fn msg(msg: Msg) -> Self {
        Effects::one(Effect::Msg(msg))
    }
    pub fn future(future: EffectFuture) -> Self {
        Effects::one(Effect::Future(future))
    }
    pub fn msgs(msgs: Vec<Msg>) -> Self {
        Effects::many(msgs.into_iter().map(Effect::from).collect())
    }
    pub fn futures(futures: Vec<EffectFuture>) -> Self {
        Effects::many(futures.into_iter().map(Effect::from).collect())
    }
    pub fn unchanged(mut self) -> Self {
        self.has_changed = false;
        self
    }
    pub fn join(mut self, mut effects: Effects) -> Self {
        self.has_changed = self.has_changed || effects.has_changed;
        self.effects.append(&mut effects.effects);
        self
    }
}

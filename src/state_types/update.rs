use crate::state_types::messages::Msg;
use crate::state_types::Effects;

pub trait Update {
    fn update(&mut self, msg: &Msg) -> Effects;
}

pub trait UpdateWithCtx<Ctx> {
    fn update(&mut self, ctx: &Ctx, msg: &Msg) -> Effects;
}

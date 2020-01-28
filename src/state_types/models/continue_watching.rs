use crate::constants::CATALOG_PREVIEW_SIZE;
use crate::state_types::models::ctx::library::LibraryLoadable;
use crate::state_types::models::ctx::Ctx;
use crate::state_types::msg::{Internal, Msg};
use crate::state_types::{Effects, Environment, UpdateWithCtx};
use crate::types::LibItem;
use lazysort::SortedBy;
use serde::Serialize;

#[derive(Default, Debug, Clone, Serialize)]
pub struct ContinueWatching {
    pub lib_items: Vec<LibItem>,
}

impl<Env: Environment + 'static> UpdateWithCtx<Ctx<Env>> for ContinueWatching {
    fn update(&mut self, ctx: &Ctx<Env>, msg: &Msg) -> Effects {
        match msg {
            Msg::Internal(Internal::LibraryChanged) => {
                lib_items_update(&mut self.lib_items, &ctx.library)
            }
            _ => Effects::none().unchanged(),
        }
    }
}

fn lib_items_update(lib_items: &mut Vec<LibItem>, library: &LibraryLoadable) -> Effects {
    let next_lib_items = match library {
        LibraryLoadable::Ready(lib_bucket) => lib_bucket
            .items
            .values()
            .filter(|lib_item| lib_item.is_in_continue_watching())
            .sorted_by(|a, b| b.mtime.cmp(&a.mtime))
            .take(CATALOG_PREVIEW_SIZE)
            .cloned()
            .collect::<Vec<LibItem>>(),
        _ => vec![],
    };
    if next_lib_items.ne(lib_items) {
        *lib_items = next_lib_items;
        Effects::none()
    } else {
        Effects::none().unchanged()
    }
}

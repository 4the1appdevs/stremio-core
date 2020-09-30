use crate::models::common::{
    descriptor_update, eq_update, DescriptorAction, DescriptorLoadable, Loadable,
};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionLoad, Internal, Msg};
use crate::runtime::{Effects, Env, UpdateWithCtx};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    pub transport_url: Url,
}

#[derive(Default, Serialize)]
pub struct AddonDetails {
    pub selected: Option<Selected>,
    pub addon: Option<DescriptorLoadable>,
}

impl<E: Env + 'static> UpdateWithCtx<Ctx<E>> for AddonDetails {
    fn update(&mut self, ctx: &Ctx<E>, msg: &Msg) -> Effects {
        match msg {
            Msg::Action(Action::Load(ActionLoad::AddonDetails(selected))) => {
                let selected_effects = eq_update(&mut self.selected, Some(selected.to_owned()));
                let addon_effects = match ctx
                    .profile
                    .addons
                    .iter()
                    .find(|addon| addon.transport_url == selected.transport_url)
                {
                    Some(addon) => eq_update(
                        &mut self.addon,
                        Some(DescriptorLoadable {
                            transport_url: selected.transport_url.to_owned(),
                            content: Loadable::Ready(addon.to_owned()),
                        }),
                    ),
                    None => descriptor_update::<E>(
                        &mut self.addon,
                        DescriptorAction::DescriptorRequested {
                            transport_url: &selected.transport_url,
                        },
                    ),
                };
                selected_effects.join(addon_effects)
            }
            Msg::Action(Action::Unload) => {
                let selected_effects = eq_update(&mut self.selected, None);
                let addon_effects = eq_update(&mut self.addon, None);
                selected_effects.join(addon_effects)
            }
            Msg::Internal(Internal::ManifestRequestResult(transport_url, result)) => {
                descriptor_update::<E>(
                    &mut self.addon,
                    DescriptorAction::ManifestRequestResult {
                        transport_url,
                        result,
                    },
                )
            }
            _ => Effects::none().unchanged(),
        }
    }
}

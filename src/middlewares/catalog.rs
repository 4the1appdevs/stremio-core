use crate::state_types::*;
use crate::types::*;
use futures::{future, Future};
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Default)]
pub struct CatalogMiddleware<T: Environment> {
    pub env: PhantomData<T>,
}
impl<T: Environment> CatalogMiddleware<T>
where
    T: Environment,
{
    pub fn new() -> CatalogMiddleware<T> {
        CatalogMiddleware { env: PhantomData }
    }
    fn for_catalog(&self, addon: &AddonDescriptor, cat: &ManifestCatalog, emit: Rc<DispatcherFn>) {
        // @TODO use transport
        // @TODO: better identifier?
        let url = addon.transport_url.replace(
            "/manifest.json",
            &format!("/catalog/{}/{}.json", cat.type_name, cat.id),
        );
        emit(&Action::CatalogRequested(url.to_owned()));
        let fut = T::fetch_serde::<CatalogResponse>(url.to_owned()).then(move |res| {
            emit(&match res {
                Ok(resp) => Action::CatalogReceived(url, Ok(*resp)),
                Err(e) => Action::CatalogReceived(url, Err(e.description().to_owned())),
            });
            future::ok(())
        });
        T::exec(Box::new(fut));
    }
}
impl<T> Handler for CatalogMiddleware<T>
where
    T: Environment,
{
    fn handle(&self, action: &Action, emit: Rc<DispatcherFn>) {
        // @TODO: match on CatalogLoad action
        if let Action::WithAddons(addons, _) = action {
            for addon in addons.iter() {
                let viable_catalogs = addon.manifest.catalogs
                    .iter()
                    .filter(|cat| cat.extra_required.is_empty());
                for cat in viable_catalogs {
                    self.for_catalog(addon, cat, emit.clone());
                }
            }
        }
    }
}

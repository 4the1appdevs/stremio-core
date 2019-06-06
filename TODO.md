## DONE

* put in web
* decided to allow creating separate state containers, and do that for separate sections (board, player, etc.)
* all the basics for addons: a Vec of AddonDescriptors
* decided that there'd be no events out, just a pipeline of actions and middlewares
* basic grouping catalogs: CatalogsFiltered{ params: ...,  }, CatalogsGrouped; Actions would need some work
* CatalogsGrouped
* figure out how the player will be integrated: as a middleware that wraps around the JS (wasm-bindgen makes this easy)
* middleware model; basic rules: actions go through; user dispatches from the beginning; each middleware has one input, one output; 
* web: build a proper example with fetch()
* figure out actions polymorphism: we need to be able to easily match, serialize and etc.; for now, monolithic list of actions is OK
* think whether this whole wasm-bindgen thing violates my own philosophy against bindings: nah, it's OK
* Handler trait (`impl state_types::Handler for UserMiddleware`)
* fix middleware async shit
* environment: decide how to do the data structure: back to Traits?
* environment: implement `fetch_serde<T>`: easier ergonomics
* learn more about futures: https://aturon.github.io/blog/2016/08/11/futures/ (select, join, `or_else`, map)
* race condition protection: CatalogReq, CatalogResp matching
* go through all routes.js and figure out how we'll begin loading them
* assign time to Nikola to work on this (~1-2 months)
* make the catalogs work: middlewares: UserMiddleware (dummy-ish) and CatalogMiddleware
* remove reqwest from the core
* web environment implementation, err handling (JSON parsing, etc.)
* clippy
* reducer multiplexer handler; or just a single StateContainer wrapper, and then the user must construct a compound state container themselves; also, we have to remove the NewState actions with the full state, and make it only a notification; .dispatch of the container should return boolean whether is changed
* state container will have IDs so that Actions can target them
* CatalogsGrouped: we only wanna render the first ~30
* find the reason for calls being slow: `get_state` takes ~50ms; optimized it by reducing the amount of data
* Environment: basic storage
* Optimization: ability to subscribe with a whitelist; for actions not matching the whitelist, subscribe only to the *occurrence*, so that we can manually `get_state()` if needed at the end of the tick (`setImmediate`)
* security: deserializing items with serde should impose a limit on String length (e.g. name, poster, etc.)
* environment: storage err handling
* SPEC: decide if a separate resource will be used for library/notifications; a separate return type (.libItems rather than .metas) is a must; DONE: seems it must be a catalog, otherwise it breaks the semantics of manifest.catalogs; we will restrict it via extraRequired
* Stream: new SPEC; we should have ways to filter streams too (e.g. HTTP + directly playable only)
* think whether stateful middlewares can be eliminated or mitigated with some memoization-inspired pattern
* `get_state` is very slow: it takes a lot of time for large-ish amounts of data: investigate & open a github issue; the specific thing that's slow is whether we return the data; the reason was the TextEncoder polyfill
* refactor: error handling: consider making an enum that will hold JsValue or other error types; see https://www.youtube.com/watch?v=B5xYBrxVSiE 
* requests: instead of the builder, use ::get(...) or ::post()
* decide whether the UserM will just pass descriptors or transports; decided on descriptors
* environment: `fetch_serde` should support advanced HTTP requests: https://developer.mozilla.org/en-US/docs/Web/API/Request/Request; just use https://github.com/DenisKolodin/yew/blob/fdb9acbd014c5178b6881faef0874495ca49e63f/src/services/fetch.rs#L14 (http::Request or a reqwest::Request);
* design decision on reacting on addon installs/uninstalls is: we don't, but issuing a new Load will make the container react on it; and we should always issue new Load's when going to pages
* look into use Into<> to get rid of some .into()'s ?
* types/addons/{mod,manifest}.rs: Descriptor, ManifestCatalog, ManifestResource, ResourceRef, AggrRequest, Extra, Request; RequestHash can be used to match the responses
* CatalogsGrouped to receive some info about the addon (from the manifest): this can be done with the new refactor where we'd use `action_load.plan()` directly in the reducer (at this point we can access addons too)
* do we want to add the ability for an addon to update it's results? it could become relatively elegant with AddonResp: no for now, but it can be done easily
* refactor: AddonRequest -> AddonRequests, since we want to guarantee preserved order of requests; or rather, drop AddonRequests/CatalogRequest entirely, and just expand WithAddons(addons, ...) plus the action `get_addon_request` directly in the reducer; that will also drop `req_id` (hash of ResourceRequest?)
* refactor: perhaps we can use Load(Target), where Target is an enum, and then wrap it in LoadWithUser(user, addons, Target) - if Load is the only place we need addons; we won't need Box<> and we can pattern match
* decide how do we wanna do CatalogsFilteredWithPreview: whether we wanna do it at all, or just have CatalogFiltered always return MetaItem; DECISION: we will simply represent a page of MetaItem, therefore we don't need anything else for previews; also, paging would be done through `extra`, so a new `Load` will have to be sent
* storage: set should take Option<T>
* more detailed errors on deserialize (environment?)
* optimization: optimize the reducers by avoiding copying data on each iteration: can't happen for now, conceptually difficult
* optimization: actually, that worked out with inner Rc<>
* refactor: generic AddonResponse (currently uses CatalogResponse)? use an untagged enum
* https://github.com/Stremio/stremio-aggregators/blob/master/lib/isCatalogSupported.js
* AddonM: extra
* extra: advanced notation implemented
* refactor: enum representations in serde
* addonM: given a `transport_url`, FromAddon will try to find the addon in the collection, to possibly apply `flags.stremioAuth` or `flags.transport`; of course, it doesn't need to find it, `transport_url` is sufficient to request; or, it should just carry the flags; **DECISION:** neither, `stremioAuth` is just put on hold for now, see https://github.com/Stremio/stremio/issues/407
* graph everything, the entire stremio architecture, including core add-ons and such
* implement UserM; think of how (or not to?) to mock storage in the test; LoadWithUser(user, addons, ...)
* UserM: figure ot loading step; perhaps always do the load with a future and do everything in a .then(), but memoize it
* construct `AddonHTTPTransport<E: Environment>` and give it to the interested middlewares; introduce a long-lived transport; addon transports can have FromStr trait?
* UserM: actions related to the user: Login, Logout, SignUp; PullAddons, PushAddons; PullUser, PushUser (?)
* UserM: how to protect from responses from previous user; RESOLVED: simple check with the current `auth_key` will suffice
* UserM: refactor addon actions into ActionAddon, UserOp should be renamed to ActionUser
* consider memoization/resetting
* TransportUrl type, safety and parsing; ensure malformed ones cant crash the program; they can't, we handle the case
* UserM: uninstall/install addons for the user, sync their collection
* semver check for manifest.json
* APIRequest/APIResponse should be enums? that enum should have a method to get the string name; easily get the api request, and then universally handle the response Error case
* UserM: proper err handling
* userM: key should be in the enum
* refactor: load first, UserStorage to convert Action -> request; userStorage to have current_auth_key(), split into files
* userM: all of the user actions should do a `load()` first
* UserM: implement the actions; consider matching them against API calls (action, call path, data structure)
* UserM: Pushaddons/PullAddons
* UserM: AddonsChanged/UserChanged actions
* bug: AddonsChanged/AddonsChangedFromPull fired before storing
* since a lot of things are asynchronous, perhaps we should have a guard; the things to think about are: addon set hash, addon ID, user ID, etc.; RESOLVED
* environment: consider allowing a dynamic instance, esp for storage; RESOLVED: No; everything can be done statically
* architecturally, can we get away with not contacting the streming server in the state container?; YES, and we should; server should be contacted by the players and settings UI only
* bug: manifest.resources loses it's properties when serialized/deserialized; shorthand should always be serialized as shorthand
* refactor: mod.rs on `state_types` and types shouldn't glob export everything
* learn about error downcasting and how we can use it
* decide on all the settings: which ones are kept where
* design flaw: the player is supposed to get the URL to the video itself (from Stream), but then it needs to pull /subtitles/ from the addon system; could be done by wrapping some messages in the state container, but maybe there's a better way? - WILL BE done through an event for playback started emitted by the implementation, that contains an `opensubHash`
* AddonM: transport type recognizer
* Stream type, .source
* AddonM: legacy transport
* Stream type: full spec
* consider full router in e example
* legacy transport: (and all transports): manifest retrieval
* AddonM: AddonTransport trait, .get(), .manifest(); http addons will be constructed with a URL, while lib/notif addon directly as something that implements AddonTransport
* refactor: Chain should not have a final callback
* refactor: FinalHandler/ContainersHandler in place of ContainerHandler, which will contain the final callback
* Actions should not contain final stuff, FinalHandler should take it's own type
* reworked Container API: Container struct needs to be mutable now; ContainerHolder handles interior mutability; the ContainerInterface trait assumes interior mutability
* API types: SuccessResponse should be (de)serialized as `{success: true}`
* container might be a trait with default methods; that way, you can construct them with args; eliminates mutability too
* try to make a UI with conrod (https://github.com/tokio-rs/tokio-core/issues/150)
* implement a basic CatalogsFiltered
* cataloggrouped: consider dropping the Arc and just copying; measure the performance, and keep in mind cases with more groups; turns out, Arc is actually fastest: https://gist.github.com/Ivshti/7ddf0fa6c7d50b5211d8f771241f64ab
* test for CatalogFiltered
* Load to be able to target particular containers; ContainerMuxer
	it will have to remmeber it's last Load itself
	filter Loads when we send a load to a container
	downcast from the muxer?
	emit a ref to &ContainerInterface with NewState; that can be downcast (this will probably need Rc<RefCell)
* refactor: figure out some identifier that links the Load to the actual end container
* state container: catalogfiltered should be split by pages
* implement a Streams container; should be split by addons
* LibItem struct
* LibItem struct: deserialize an emtpy string as None
* rename to stremio-core
* Video struct
* manifest: make the extra field for catalogs private, and have `get_extra()` function that returns in a uniform notation; use that
* actions: consider #[serde(skip)] for many things; rationale: shaves off binary size (cause of generating serialize/deserialize); also enforces correct usage
* Optimization: web example takes 350ms to load the JS/babel/all webpack shit; try without webpack; TRIED WITHOUT WEBPACK; but it turned out the time waste was in third party extensions! Always benchmark without extensions! ; now takes around ~50ms to load everything with cache
* consider alternative Actions; where it's split in Input/Mid/Output but it can be constructed (instantiated) and deconstructed (matched against) easily
	Msg: Action, Internal, Event
	split into msg.rs and actions.rs
* research libitempreview memory use: it uses around 300MB for a million of items, so it's fine to just keep an in-memory undex
* library addon - handles interior mutability (Arc + Mutex); implement Handler and AddonInterface
* AddonTransportMuxer; construct with a BTreeMap of <TransportUrl, AddonInterface>; ContextM will emit LibraryAddonUpdated(interface) or SetInternalAddon({addon,transport_url})
* Detect transport type function, Result<dyn AddonInterface>; to return the library addon interface; NOT NEEDED: see Addon middleware (it holds `extra_addons`)
* PROBLEM: CatalogFiltered requires all rows to be MetaPreview; continue watching is LibItem
	either use a separate container (original plan) <- DOING THIS
	or make the container polymorphic (security implications?)
	or map to MetaPreview
* DESIGN: middlewares vs elm-like Cmd? it doesn't seem we can do eveyrthing with Cmd (e.g. Context) 
* https://llogiq.github.io/2017/06/01/perf-pitfalls.html if we ever need optimizations; we do `to_owned` quite a lot, maybe some of those can be avoided; `Cow<>` sounds good too for large collections and etc.; we likely won't
* spec: notifItems: rethink that spec, crystallize it
* decide: complex async pieces of logic: open, detectFromURL, openMedia; those should be a middleware or just separate async functions; detectFromURL/openMedia are user-agnostic, but open is not; if it's an async function used internally by the middleware, it's still OK cause we won't make the stream requests again if we go to the UI (cause of the memoization); decided: should be separate functions `recommend_open` and`recommend_open_media`, that get invoked by some middleware (prob ContextM)
* refactor: consider splitting Environment into Storage and Fetcher; and maybe take an extra AddonsClient in; won't do it: there's no point of it, also all methods are intended to be static cause that way we ensure it's a singleton


## TODO

* Optimization: web version: CI to use a headless browser to measure load times
* integration testing plan; some e2e tests (e.g. player, settings, login/logout, libitem sync) will be nice

* all issues to github; take into account iOS notes too

* document PlayerPreferences and etc.; binging, saving library item state, marking episodes watched, marking notifications seen
	mode of operation: either with a libitem, or without one

* Open (`recommend_open`) should always pull the libitem first; this is a UX improvement, ensures we do not lose our playing status on another device if we just click before syncing on one device

* contextM: `last_modified` for addons, prevent race conditions by updating `last_modified` each time we modify; consider sequence numbers too

* document loopback actions (implicit input): `AddonsChanged->PushAddons` (if there's a conn), (as a result of Open) `ProposeLoad -> Load`; `ProposeWatchNext -> Open`; also those that are results of OpenMedia, InstallAndOpenAddon

* environment: the JS side should (1) TRY to load the WASM and (2) TRY to sanity-check the environment; if it doesn't succeed, it should show an error to the user

* ?addonOpen/InstallAndOpenAddon: another async action

* UX: we should make it so that if a session is expired, we go to the login screen; this should probably be in the app
* player: implement playerPreferences and defaults behavior: picking a default subtitle/audio track; for audio, the logic should try to select your preferred language
* ensure that every time a network error happens, it's properly reflected in the state; and the UI should allow to "Retry" each such operation
* figure out pausing on minimize/close; this should be handled in the app; probably like this: when closing/minimizing the window, pause if state is playing
* JS Side: All errors or warnings that come as actions should be reported to sentry; including UserMiddlewareFatal
* fuzzing all addons: load all addons (addonscollection, addonsofficialcollection), request all catalogs, then all metas and then all streams; that way, we find if anything returned by the addons is unserializable by the types crate


---------

## Actions from the user

Load
	works for opening catalogs/detail/load/search
Unload
TryLogin
TrySignup
TryOpen libItem|metaItem intent videoId?
TryOpenURL
LibAdd type id
LibRemove type id
LibRewind type id
LibDismissAllNotifs type id
LibSetReceiveNotifs type id
LibMarkWatched type id
LibMarkVideoWatched type id videoId true|false
TryAddonRemove
TryAddonAdd
TryAddonOpenURL - consider if this should be merged with TryOpenURL
NotifDismiss id
PlayerSetProp
PlayerCommand

## Player (player spec wrapper) middleware

LibItemPlayerSave (will be consumed by library addon middleware)
alternatively, LibItemSetTime/LibItemSetVideoID
... everything from the player spec
ProposeWatchNext
this should also start loading the subtitles from addons and such
all/most player actions should carry context in some way (stream, or at least stream ID, and maybe video ID, item ID)
this middleware uses Storage to persist PlayerPreferences (volume, subtitles, subtitles size, etc.); we must keep preferences for last N watched `(item_id, video_id)`
the algo to do this is simple; when we play something, we bump it to the end of the array; when we need to add something, we add to the end and pop from the middle (if `len>N`)
This should save the selected `(video_id, stream)` for the given `item_id`), when we start playing
we also need to load `meta` to be able to `ProposeWatchNext` (meant to be handled by asking the user or by implicit input)

upon a LoadPlayer, we load the PlayerPreferences send a command to the player to select the previously selected subtitles ID (if any)
if we get a AddonResp for subtitles, we send a addExtraSubtitles command
if we get an AddonsFinished, we try to select previously selected ID as well (if we haven't succeeded in doing so already)
if we don't have a selected ID at all, we should go with the default language

for player messages, it would be very nice if we had some identifier of the current stream, so that we can discard messages coming from a previous stream

@TODO NOTE: since we need easy immediate access to the preferences, memoization is the wrong pattern here and we need statefulness

Please note, there'd be no player reducer for now, as all of the state updates come in the form of player `propValue` or `propChanged` actions, which is very simple to reduce

all of the state: PlayerImplInstance, PlayerPreferences, ItemId/VideoId/MetaDetailed/StreamId

figure out state container + player impl + subtitles hash (requesting /subtitles)
	can be done by the player impl emitting an event for playback started with extra optional prop `opensubHash`

### More thoughts on Player

It will be very stateful, so it should be named Actor perhaps?

It will "enclose" the player implementation, sending all control msgs into it and taking all observe messages along the chain

It’s job will be to load the library item, mutate it as the player advances; when the player starts playing it should request subtitles from the addon system (for this, it should keep a copy from all subtitle addons that’s changed on every LoadWithUser(Player...)) ) and do addExtraSubtitles and perhaps set selected subtitles when applying PlayerPreferences; 
Call addExtraSubtiles with a concatted result from each addon; the result actually starts with the `stream.subtitles` (if any) 

It should also mutate and persist PlayerPreferences itself

Also the Load actions that are translated to Aggr requests will be responsible for loading the meta when the player loads 

Since it knows the current library item, it can also attach a “session ID” to player messages are they’re passed along; this could be used in the reducers to prevent races 

----

# Routes

### ?apiURL

overrides the API URL
this will simply tweak the Environment

### ?addonURL=url

prompts the user to install an add-on or a collection
this should dispatch Actions::OpenAddonURL

### ?addonURLForce=Url

force adds the given add-on or collection of add-ons; dispatch Actions::InstallAddonURL
@TODO consider the security aspect of this

### /board

Dispatch LoadCatalogsGrouped(0) -> AddonAggrReq(Catalogs())

### /discover/:type/:addonID/:catalogID/:filters?&preview=ID

Dispatch LoadCatalogsFiltered(1, type, addonID, catalogID, filtered) -> AddonAggrReq(OfResource("catalog", type, catalogID, filters)) but match it only against the addon with addonID

@TODO addonTransportURL and OfAddon instead of addonID; more concise, allows URLs to work for other pepole too, and simplifies the middleware

If, for some reason, we use a `type` that's not available, the particular addon will return an error, which will be transformed into Loadable::Message and handled elegantly 

@TODO routing problem: if /discover is opened, we need to auto-select some (type, catalog, filters); we might just hardcode Cinemeta's top and always go to that

### /library/:type

Dispatch Load(CatalogFiltered(type, "stremio://library", "library", { library: 1 })) -> AddonAggrReq(OfAddon("stremio://library", "catalogs", type, "library", { library: 1 })) but match against library addon

If we do addonTransportURL+OfAddon, and we save the last selected `type` in the UI, If, for some reason, we use a `type` that's not available, the particular addon will return an error, which will be transformed into Loadable::Message and handled elegantly 

NOTE: the library addon manifest will include catalogs only for the types of the items the user has


### Notifications (not a route, but a popover)

Dispatch Load(Notifications) -> AddonAggrReq(Catalogs({ notifs: 1 }))

### /addons/:category/:type?

Category is Official, ThirdParty, Installed

Dispatch Load(AddonCatalog(category, type)) -> middleware loads latest collection of the given category and filters by type 

### /player/:type/:id/:videoId/:streamSerialized

Dispatch Load(Player(type, id, videoId, streamSerialized)) -> this will trigger many things, one of them AddonAggrReq(OfResource("meta", type, id))
	another one will be to load the libitem/notifications
	the player middleware should also request subtitles from the add-on system (AddonAggrReq(OfResource("subtitles", meta, id)))
	the player middleware should also keep an internal state of what the player is doing, and persist libitem/last played stream

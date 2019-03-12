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


## TODO

* consider full router in the example


* legacy transport: (and all transports): manifest retrieval
* AddonM: AddonTransport trait, .get(), .manifest(); http addons will be constructed with a URL, while lib/notif addon directly as something that implements AddonTransport

* UserM: `last_modified` for addons, prevent race conditions by updating `last_modified` each time we modify; consider sequence numbers too
* UserM: upgrade addons when doing the first pull

* start doing documentation comments

* UserM: plug in a built in addon (LibraryAddon)
* UserM: because of the settings, we might need to rename it to ContextM/LoadWithCtx

* addon catalog reducer, actions; handle loading collections in the addonM
* AddonM: caching: statefulness can be mitigated by using a memoization where the addon transport `get` would return the same results if invoked with the same args again; however, this needs to be out of the transport impl and needs to be explicit
* UserM: mock storage and tests

* implement CatalogsFiltered

* API types: `()` should be (de)serialized as `{success: "true"}`
* consider ResourceRef having ResourceType

* test if addoncollection can be parsed and understood, once the middleware(s) can retrieve collections
* addon catalog reducer

* basic state: Catalog, Detail; and all the possible inner states (describe the structures); StreamSelect
* tests: Chain, Container, individual middlewares, individual types
* Load to be able to target particular containers
* start implementing libitem/notifitem addon
* load/unload dynamics and more things other than Catalog: Detail, StreamSelect

* Video type, detailed meta

* environment implementations: return an error related to the HTTP status code, if it's not 200

* document loopback actions (implicit input): `AddonsChanged->PushAddons` (if there's a conn), (as a result of Open) `ProposeLoad -> Load`; `ProposeWatchNext -> Open`; also those that are results of OpenMedia, InstallAndOpenAddon


* refactor: separate crates: types, `state_types`; the point of that is to not install any unneeded deps

* Trait for meta item and lib item; MetaPreview, MetaItem, MetaDetailed
* stuff to look for to be re-implemented: syncer, libitem/notifitem addons, discover ctrl, board ctrl, detail ctrl
* environment: the JS side should (1) TRY to load the WASM and (2) TRY to sanity-check the environment; if it doesn't succeed, it should show an error to the user
* complex async pieces of logic: open, detectFromURL, openMedia; those should be a middleware or just separate async functions; detectFromURL/openMedia are user-agnostic, but open is not; if it's an async function used internally by the middleware, it's still OK cause we won't make the stream requests again if we go to the UI (cause of the memoization)
* ?addonOpen/InstallAndOpenAddon: another async action
* opening a file (protocol add-ons to be considered)
* crates: stremio-web-environment (only the Environment), stremio-state-ng-web (general API that is exported to JS via bindgen)
* we should make it so that if a session is expired, we go to the login screen; this should be in the app
* think of how to do all edge cases in the user, such as pre-installing add-ons (implicit input)
* behaviorHints - pair (key, val)?
* environment (web): separate crate, also can we avoid the double deserialization on `fetch_serde`?
* when playing AND when opening the detail page, we should augment the libItem with meta if it's not already (trigger the updateLibItem action only if this would actually change the libitem)
* when saving the last stream, save the whole object but compressed
* player: implement playerPreferences and defaults behavior: picking a default subtitle/audio track; for audio, the logic should try to select your preferred language
* player: we migh benefit from refactoring the save/load stuff from userM into memoizedStorageSlot and using that
* ensure environment caches are in place via the service worker (web)
* spec: notifItems: rethink that spec, crystallize it

* consider: flag `is_in_lib` for catalog items; could just work for Discover by having another CatlaogFiltered showing ("meta", type, id) from the lib addon
* https://github.com/woboq/qmetaobject-rs based UI; needs reqwest (or someting else) async requests
* libitem/notifitem: https://developers.cloudflare.com/workers/kv/ https://blog.cloudflare.com/cloudflare-workers-as-a-serverless-rust-platform/
* think of whether this could be used with the Kodi codebase to make stremio embedded
* all the cinemeta improvements this relies on: e.g. behaviorHints.isNotReleased will affect the Stream view
* ensure that every time a network error happens, it's properly reflected in the state; and the UI should allow to "Retry" each such operation
* figure out pausing on minimize/close; this should be handled in the app; probably like this: when closing/minimizing the window, pause if state is playing
* when you go to player/detail and there doesn't appear to be a supported addon for the /meta/ request, show an error to the user (+test for that?)
* refactor: consider splitting Environment into Storage and Fetcher; and maybe take AddonsClient in
* document item type agnostic behavior (detail page)
* https://blog.cloudflare.com/cloudflare-workers-as-a-serverless-rust-platform/
* JS Side: All errors or warnings that come as actions should be reported to sentry
* more manual/automated tests: ensure that when UserMiddlewareFatal happens, it is reported
* fuzzing all addons: load all addons (addonscollection, addonsofficialcollection), request all catalogs, then all metas and then all streams; that way, we find if anything returned by the addons is unserializable by the types crate
* Discover UI: if we've opened an addon that is not installed, there should be an "This addon is not installed. Install now?" notification on top

* BACKEND: notifitem generation needs to be reduced (10 per item, max ~300)
* lib/notif addon: gzip everything?

* https://llogiq.github.io/2017/06/01/perf-pitfalls.html if we ever nede optimizations; we do `to_owned` quite a lot, maybe some of those can be avoided; `Cow<>` sounds good too for large collections and etc.

* refactor: consider using [Url](https://docs.rs/url_serde/0.2.0/url_serde/) for `transport_url`, addon `logo`/`poster`, meta `poster`/`logo`/`background`, stream `url`/`external_url`


work estimation, hours: 24 userM, 12 addonM + transport, 10 legacy transport, 8 refactors, 3 catalogFiltered, 6 detail/streamselect, 24 lib/notif addon, 8 playerM, 8 open, 8 openMedia, 12 others, 10 tests: 127 = 13 weekends assumming 10 hours per weekend = 6 weeks

example pipeline:
LoadCatalogs => this will change the state of the `catalogs` to `Loading`
LoadFromAddons(addons, 'catalog') => emitted from the UserMiddleware
many AddonRequest(addon, 'catalog')
many AddonResponse(addon, 'catalog', resp) => each one would update the catalogs state

---------

## Universe actions: 
UserDataLoad
InitiateSync (or separate events; see https://github.com/Stremio/stremio/issues/388)
BeforeClose
SettingsLoad
TryStreamingServer (this will try connecting to the streaming server, as well as probing it's settings and version)
NetworkStatusChanged
WindowStateChanged (playerM will react on that to pause the player if the setting is true)

## Actions from the user

Load reducerType reducerId ...
	works for opening catalogs/detail/load/search
	reducerType is needed so that middlewares know to react; we can remove that by instructing the middlewares which reducerIds they should care about
	the library middleware will try to attach a selected type if there isn't one
	the catalogs middleware should dispatch msgs intended for the grouped or filtered reducers; the search should go through to the grouped; we will use separate reducerIds for board/search
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

## Settings middleware:

It will persist settings in storage


figure out whether we need a settings container/middleware in stremio-state-ng; check list of settings, check which ones are user synced
	think which ones can actually be storred as addon flags
	ensure there's a simple, usable and global pattern for settings
	also think about which ones have to be applied in the state container itself
	https://github.com/Stremio/labs/issues/20

### All Settings:

Settings can be stored in 4 places: localStorage, user, addonCollection, server

language: currently user, considering localStorage
autoplay_next_video: localStorage

player_mpv_hwdec: localStorage; considering Shell
player_mpv_cache_size: localStorage; considering Shell
player_mpv_separate_window: localStorage; considering Shell
player_esc_exits_fullscreen: localStorage
player_pause_on_minimize: localStorage
player_use_external: localStorage

player_default_external: localStorage; consider introducing a new setings location "shell", and store this one there

server_cache_location: server
server_cache_size: server
server_torrent_profile: server

server_fallback_url: localStorage

show_ep_synnopsis: localStorage; could be addonCollection (cinemeta flag)
subtitles_language: localStorage
subtitles_size: localStorage
subtitles_fgcolor: localStorage
subtitles_outline_color: localStorage
subtitles_background: localStorage

autoplay_addon: addonCollection: this will be an `autoplay` flag for each addon in the collection; that, combined with the order of the addons in the collection, will determine autoplay behavior; the flag may contain details, such as `{ onlyGroups: ["hd"] }` (more about groups: https://github.com/Stremio/stremio/issues/524)

Notifications will be handled through the User Panel


## User middleware:
...

consider: UserChanged / AddonCollectionChanged

UserOp (Login, Signup, Logout, PushAddons, PullAddons)
RemoveAddon/InstallAddon -> only does things locally and emits AddonCollectionChanged; the app should invoke PushAddons if it's online

error origins
* .load() failed: unrecoverable: UserFatal
* pulling/pushing addons failed (non fatal): UserOpWarning(action, err)
* Login/Signup failed (needs user feedback): UserOpError(action, err); needs to preserve API error though

can be generalized to EnvError, APIError (it will be nice if we can distinct between fetch and storage errors)

All errors should be sent to Sentry, and all warnings should be displayed to the user, but we should NOT attempt to do stuff when the user is offline (should not attempt to sync addons and etc.)

Load -> LoadWithUser(Option<user>, addons, ...)

how to protect against race conditions where the responses of requests made with prev authKey arrive? maybe just take a `to_owned()`
of the auth key in the beginning, and only persist if the auth key matches

how/whether to trigger pull addons on user login? sounds like we should, and we should treat it as one operation


## AddonAggr
transforms LoadWithUser(dyn AddonReq) (any action implementing the AddonReq trait), and then AddonAdded/AddonRemoved into -> AddonRequest + AddonResponse
this can be universally used by a lot (see below)

AllAddonRequestsFinished(original action) - wrap the original action


@TODO should we have an action for ALL pending addon requests being done?

## Detail middleware
is it even needed, if we have a completely stateless design?
Think of how to architect the StreamsPicker; it might need to be a separate reducer; in this case the middleware must be renamed to "DetailAndStream"


## Player (player spec wrapper) middleware:
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

## Library/Notifications middleware:

It's job is to handle actions that intend to change LibraryItem/NotifItem objects, do those changes/syncs and emit whats going on
ItemUpdated(ID, we have to have a result here, whether the libitem synced successfully)
SyncCompleted

The reducer should handle ItemUpdated(...)

Final reducers will be catalog, library, notifications, detail, player, settings, intro

player reducer should accurately reflect states like subtitles (from addons) or subtitle files (vtt) loading

### continue watching

show an item if `item.state.video_id != item._id` and `timeOffset == 0` and there is a `video_id`; the goal is to show series that you have a next ep of
resetting state should work like this: if there is no next video, reset `video_id` to null; 
first sort by the date of the first notification, second sort by lastWatched; the goal here is to show items with notifications (if overallTimeWatched > 0)

the classic reason to show an item in continue watching is if `(!removed || temp) && timeOffset > 0`; extend that by also requiring `timeOffset < duration*0.95`

All of this should be defined in `lib_item.is_in_continue_watching()`

## another middleware for open, openMedia, openAddonURL

@TODO

## Analytics sink:

needs to take installationID as an arg

every event needs to have a seq number and a session


------


Initial flow to be implemented:
LoadCatalog -> (user middleware does this) WithUser(user, addons, LoadCatalog) -> AddonResponse

The reducer, upon a LoadCatalog, should .clone() the action into it's state, and then discard any AddonRequest/AddonResponse that doesn't match that

# Routes

Presumes the following reducers

0: CatalogsGrouped (for board)
1: CatalogsFilteredWithPreview (for discover); @TODO: this might be two separate reducers: CatalogsFiltered, CatalogsFilteredPreview
2: CatalogsGrouped (for search)
3: Detail
4: Streams
5: CatalogsFiltered (for library)
6: CatalogsFiltered (for notifications; could be specific: CatalogsNotifGrouped)
7: AddonCatalog
8: PlayerView
9: SettingsView
@TODO a container for Continue Watching

@TODO figure reload/force policies for all of these; for now, we'll just always load everything (naively)

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

### /detail/:type/:id/:videoID?

Dispatch LoadDetail(3, type, id) -> AddonAggrReq(OfResource("meta", type, id))
if videoID, dispatch LoadStreams(4, type, id, videoID) -> AddonAggrReq(OfResource("stream", type, videoID)) ; this also needs to read the last selected stream from storage

The Library item and the notifications will be loaded through the AddonAggrReq(OfResource("meta", type, id)); that will match the library/notif addon, and return the results

Complex interactions such:

* marking notifs as dimissed
* marking videos as watched
* libItem: removing/adding
* libItem: changing whether we receive a notification

Since we generate all that from `AddonAggrReq(OfResource("meta", type, id))`, we should trigger a refresh somehow that overrides the memoization for the notif/library stuff


### /library/:type

Dispatch LoadCatalogsFiltered(5, type, "org.stremio.library", "library", { library: 1 }) -> AddonAggrReq(OfResource("catalogs", type, "library", { library: 1 })) but match against library addon

If we do addonTransportURL+OfAddon, and we save the last selected `type` in the UI, If, for some reason, we use a `type` that's not available, the particular addon will return an error, which will be transformed into Loadable::Message and handled elegantly 

### Notifications (not a route, but a popover)

Dispatch LoadCatalogsGrouped(6) -> AddonAggrReq(Catalogs({ notifs: 1 }))

### /addons/:category/:type?

Category is Official, ThirdParty, Installed

Dispatch LoadAddonCatalog(7, category, type) -> middleware loads latest collection of the given category and filters by type 

### /player/:type/:id/:videoId/:streamSerialized

Dispatch LoadPlayer(8, type, id, videoId, streamSerialized) -> this will trigger many things, one of them AddonAggrReq(OfResource("meta", type, id))
	another one will be to load the libitem/notifications
	the player middleware should also request subtitles from the add-on system (AddonAggrReq(OfResource("subtitles", meta, id)))
	the player middleware should also keep an internal state of what the player is doing, and persist libitem/last played stream

### /calendar

@TODO
CalendarMIddleware needs to get the calendar from the stremio-web-services

### /intro

@TODO

### /settings

We need ot load the existing settings (settingsmiddleware might hold them anyway)
and we have to try to connect to the streaming server

@TODO

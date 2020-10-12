use crate::types::library::{LibraryItem, LibraryItemBehaviorHints, LibraryItemState};
use crate::types::resource::PosterShape;
use crate::unit_tests::serde::default_token_ext::DefaultTokens;
use chrono::prelude::TimeZone;
use chrono::Utc;
use serde_test::{assert_tokens, Token};

#[test]
fn deserialize_library_item() {
    let library_item = [
        Token::Seq { len: Some(1) },
        Token::Struct {
            name: "LibraryItem",
            len: 11,
        },
        Token::Str("_id"),
        Token::Str("id1"),
        Token::Str("name"),
        Token::Str("name"),
        Token::Str("type"),
        Token::Str("type"),
        Token::Str("poster"),
        Token::Some,
        Token::Str("poster"),
        Token::Str("posterShape"),
        Token::UnitVariant {
            name: "PosterShape",
            variant: "square",
        },
        Token::Str("removed"),
        Token::Bool(true),
        Token::Str("temp"),
        Token::Bool(true),
        Token::Str("_ctime"),
        Token::Some,
        Token::Str("2020-01-01T00:00:00Z"),
        Token::Str("_mtime"),
        Token::Str("2020-01-01T00:00:00Z"),
        Token::Str("state"),
    ]
    .iter()
    .chain(LibraryItemState::default_token().iter())
    .chain([Token::Str("behaviorHints")].iter())
    .chain(LibraryItemBehaviorHints::default_token().iter())
    .chain([Token::StructEnd, Token::SeqEnd].iter())
    .cloned()
    .collect::<Vec<Token>>();
    let mut vec = Vec::new();
    vec.push(LibraryItem {
        id: "id1".to_owned(),
        name: "name".to_owned(),
        type_: "type".to_owned(),
        poster: Some("poster".to_owned()),
        poster_shape: PosterShape::Square,
        removed: true,
        temp: true,
        ctime: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
        mtime: Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0),
        state: LibraryItemState::default(),
        behavior_hints: LibraryItemBehaviorHints::default(),
    });
    assert_tokens(&vec, &library_item);
}

use crate::types::resource::StreamSource;
use url::Url;

#[test]
fn deserialize_stream_source_url() {
    let url = StreamSource::Url {
        url: Url::parse("https://url").unwrap(),
    };
    let url_json = r#"
    {
        "url": "https://url/"
    }
    "#;
    let url_deserialize = serde_json::from_str(&url_json).unwrap();
    assert_eq!(url, url_deserialize, "Url deserialized successfully");
}

#[test]
fn deserialize_stream_source_youtube() {
    let youtube = StreamSource::YouTube {
        yt_id: "yt_id".to_owned(),
    };
    let youtube_json = r#"
    {
        "ytId": "yt_id"
    }
    "#;
    let youtube_deserialize = serde_json::from_str(&youtube_json).unwrap();
    assert_eq!(
        youtube, youtube_deserialize,
        "YouTube deserialized successfully"
    );
}

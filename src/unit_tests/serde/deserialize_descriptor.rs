use crate::types::addon::{
    Descriptor, DescriptorFlags, Manifest, ManifestCatalog, ManifestExtra, ManifestExtraProp,
    ManifestResource, OptionsLimit,
};
use semver::Version;
use url::Url;

#[test]
fn deserialize_descriptor() {
    let descriptors = vec![
        Descriptor {
            manifest: Manifest {
                id: "id".to_owned(),
                version: Version::new(0, 0, 1),
                name: "name".to_owned(),
                contact_email: Some("contact_email".to_owned()),
                description: Some("description".to_owned()),
                logo: Some("logo".to_owned()),
                background: Some("background".to_owned()),
                types: vec!["type".to_owned()],
                resources: vec![
                    ManifestResource::Short("resource".to_owned()),
                    ManifestResource::Full {
                        name: "name".to_owned(),
                        types: Some(vec!["type".to_owned()]),
                        id_prefixes: Some(vec!["id_prefix".to_owned()]),
                    },
                ],
                id_prefixes: Some(vec!["id_prefix".to_owned()]),
                catalogs: vec![ManifestCatalog {
                    type_name: "type_name".to_owned(),
                    id: "id".to_owned(),
                    name: Some("name".to_owned()),
                    extra: ManifestExtra::Full {
                        props: vec![ManifestExtraProp {
                            name: "name".to_owned(),
                            is_required: true,
                            options: Some(vec!["option".to_owned()]),
                            options_limit: OptionsLimit(1),
                        }],
                    },
                }],
                addon_catalogs: vec![ManifestCatalog {
                    type_name: "type_name".to_owned(),
                    id: "id".to_owned(),
                    name: Some("name".to_owned()),
                    extra: ManifestExtra::Short {
                        required: vec!["required".to_owned()],
                        supported: vec!["supported".to_owned()],
                    },
                }],
                behavior_hints: vec![("behavior_hint".to_owned(), serde_json::Value::Bool(true))]
                    .iter()
                    .cloned()
                    .collect(),
            },
            transport_url: Url::parse("https://transport_url").unwrap(),
            flags: DescriptorFlags {
                official: true,
                protected: true,
            },
        },
        Descriptor {
            manifest: Manifest {
                id: "id".to_owned(),
                version: Version::new(0, 0, 1),
                name: "name".to_owned(),
                contact_email: None,
                description: None,
                logo: None,
                background: None,
                types: vec![],
                resources: vec![],
                id_prefixes: None,
                catalogs: vec![],
                addon_catalogs: vec![],
                behavior_hints: vec![].iter().cloned().collect(),
            },
            transport_url: Url::parse("https://transport_url").unwrap(),
            flags: DescriptorFlags {
                official: false,
                protected: false,
            },
        },
    ];
    let descriptors_json = r#"
    [
        {
            "manifest": {
                "id": "id",
                "version": "0.0.1",
                "name": "name",
                "contactEmail": "contact_email",
                "description": "description",
                "logo": "logo",
                "background": "background",
                "types": [
                    "type"
                ],
                "resources": [
                    "resource",
                    {
                        "name": "name",
                        "types": [
                            "type"
                        ],
                        "idPrefixes": [
                            "id_prefix"
                        ]
                    }
                ],
                "idPrefixes": [
                    "id_prefix"
                ],
                "catalogs": [
                    {
                        "type": "type_name",
                        "id": "id",
                        "name": "name",
                        "extra": [
                            {
                                "name": "name",
                                "isRequired": true,
                                "options": [
                                    "option"
                                ],
                                "optionsLimit": 1
                            }
                        ]
                    }
                ],
                "addonCatalogs": [
                    {
                        "type": "type_name",
                        "id": "id",
                        "name": "name",
                        "extraRequired": [
                            "required"
                        ],
                        "extraSupported": [
                            "supported"
                        ]
                    }
                ],
                "behaviorHints": {
                    "behavior_hint": true
                }
            },
            "transportUrl": "https://transport_url/",
            "flags": {
                "official": true,
                "protected": true
            }
        },
        {
            "manifest": {
                "id": "id",
                "version": "0.0.1",
                "name": "name",
                "types": [],
                "resources": [],
                "idPrefixes": null
            },
            "transportUrl": "https://transport_url/"
        }
    ]
    "#;
    let descriptors_deserialize: Vec<Descriptor> = serde_json::from_str(&descriptors_json).unwrap();
    assert_eq!(
        descriptors, descriptors_deserialize,
        "descriptor deserialized successfully"
    );
}

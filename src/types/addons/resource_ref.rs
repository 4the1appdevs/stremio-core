use percent_encoding::{percent_decode, utf8_percent_encode, NON_ALPHANUMERIC};
use serde_derive::*;
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;
use url::form_urlencoded;
pub type ExtraProp = (String, String);

// ResourceRef is the type that represents a reference to a specific resource path
// in the addon system
// It can be stringified and parsed from a string, which is used by the addon transports

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct ResourceRef {
    pub resource: String,
    pub type_name: String,
    pub id: String,
    pub extra: Vec<ExtraProp>,
}

impl ResourceRef {
    pub fn without_extra(resource: &str, type_name: &str, id: &str) -> Self {
        ResourceRef {
            resource: resource.to_owned(),
            type_name: type_name.to_owned(),
            id: id.to_owned(),
            extra: vec![],
        }
    }
    pub fn with_extra(resource: &str, type_name: &str, id: &str, extra: &[ExtraProp]) -> Self {
        ResourceRef {
            resource: resource.to_owned(),
            type_name: type_name.to_owned(),
            id: id.to_owned(),
            extra: extra.to_owned(),
        }
    }
    pub fn get_extra_first_val(&self, key: &str) -> Option<&str> {
        self.extra
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v as &str)
    }
    pub fn set_extra_unique(&mut self, key: &str, val: String) {
        let entry = self.extra.iter_mut().find(|(k, _)| k == key);
        match entry {
            Some(entry) => entry.1 = val,
            None => self.extra.push((key.to_owned(), val)),
        }
    }
    // Compare, but without considering extra
    pub fn eq_no_extra(&self, other: &ResourceRef) -> bool {
        self.resource == other.resource && self.type_name == other.type_name && self.id == other.id
    }
}

impl fmt::Display for ResourceRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "/{}/{}/{}",
            &utf8_percent_encode(&self.resource, NON_ALPHANUMERIC),
            &utf8_percent_encode(&self.type_name, NON_ALPHANUMERIC),
            &utf8_percent_encode(&self.id, NON_ALPHANUMERIC)
        )?;
        if !self.extra.is_empty() {
            let mut extra_encoded = form_urlencoded::Serializer::new(String::new());
            for (k, v) in self.extra.iter() {
                extra_encoded.append_pair(&k, &v);
            }
            write!(f, "/{}", &extra_encoded.finish())?;
        }
        write!(f, ".json")
    }
}

#[derive(Debug)]
pub enum ParseResourceErr {
    WrongPrefix,
    WrongSuffix,
    InvalidLength(usize),
    DecodeErr,
}
impl FromStr for ResourceRef {
    type Err = ParseResourceErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('/') {
            return Err(ParseResourceErr::WrongPrefix);
        }
        if !s.ends_with(".json") {
            return Err(ParseResourceErr::WrongSuffix);
        }
        let components: Vec<&str> = s.trim_end_matches(".json").split('/').skip(1).collect();
        match components.len() {
            3 | 4 => Ok(ResourceRef {
                resource: parse_component(components[0])?,
                type_name: parse_component(components[1])?,
                id: parse_component(components[2])?,
                extra: components
                    .get(3)
                    .map(|e| form_urlencoded::parse(e.as_bytes()).into_owned().collect())
                    .unwrap_or_default(),
            }),
            i => Err(ParseResourceErr::InvalidLength(i)),
        }
    }
}
fn parse_component(s: &str) -> Result<String, ParseResourceErr> {
    Ok(percent_decode(s.as_bytes())
        .decode_utf8()
        .map_err(|_| ParseResourceErr::DecodeErr)?
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_extra() {
        // We're using UTF8, slashes and dots in the ID to test if we're properly URL path encoding
        let r = ResourceRef::without_extra("catalog", "movie", "top/лол/.f");
        assert_eq!(r, ResourceRef::from_str(&r.to_string()).unwrap());
    }

    #[test]
    fn with_extra() {
        let extra = &[
            ("search".into(), "тест & z".into()),
            ("another".into(), "/something/".into()),
        ];
        let r = ResourceRef::with_extra("catalog", "movie", "top/лол.f", extra);
        assert_eq!(r, ResourceRef::from_str(&r.to_string()).unwrap());
    }

    #[test]
    fn empty_extra() {
        let extra = &[];
        let r = ResourceRef::with_extra("catalog", "movie", "kek", extra);
        assert_eq!(r, ResourceRef::from_str(&r.to_string()).unwrap());
        assert_eq!(&r.to_string(), "/catalog/movie/kek.json");
    }

    #[test]
    fn compatible_with_js() {
        let extra = &[
            ("search".into(), "the office".into()),
            ("some_other".into(), "+тест & z".into()),
        ];
        let r = ResourceRef::with_extra("catalog", "series", "top", extra);
        let js_str = "/catalog/series/top/search=the%20office&some_other=%2B%D1%82%D0%B5%D1%81%D1%82%20%26%20z.json";
        // the only difference is that stremio-core uses '+' rather than '%20'
        assert_eq!(js_str.replace("%20", "+"), r.to_string());
        // ...which we have to handle correctly
        assert_eq!(ResourceRef::from_str(&js_str).unwrap(), r);
    }

    #[test]
    fn get_extra_first_val() {
        let extra = &[
            ("search".into(), "the office".into()),
            ("foo".into(), "bar".into()),
            ("foo".into(), "test".into()),
        ];
        let r = ResourceRef::with_extra("catalog", "series", "top", extra);
        assert_eq!(r.get_extra_first_val("foo"), Some("bar"));
    }
}

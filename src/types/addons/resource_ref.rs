use serde_derive::*;
use std::fmt;
use std::str::FromStr;
use url::form_urlencoded;
use url::percent_encoding::{utf8_percent_encode, percent_decode, PATH_SEGMENT_ENCODE_SET};
pub type Extra = Vec<(String, String)>;

// ResourceRef is the type that represents a reference to a specific resource path
// in the addon system
// It can be stringified and parsed from a string, which is used by the addon transports

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ResourceRef {
    pub resource: String,
    pub type_name: String,
    pub id: String,
    pub extra: Extra,
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
}

impl fmt::Display for ResourceRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "/{}/{}/{}",
            &utf8_percent_encode(&self.resource, PATH_SEGMENT_ENCODE_SET),
            &utf8_percent_encode(&self.type_name, PATH_SEGMENT_ENCODE_SET),
            &utf8_percent_encode(&self.id, PATH_SEGMENT_ENCODE_SET)
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
    DecodeExtraErr,
}
impl FromStr for ResourceRef {
    type Err = ParseResourceErr;

    // @TODO remove .json at the end
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("/") {
            return Err(ParseResourceErr::WrongPrefix);
        }
        if !s.ends_with(".json") {
            return Err(ParseResourceErr::WrongSuffix);
        }
        let components: Vec<&str> = s
            .trim_end_matches(".json")
            .split('/')
            .skip(1)
            .collect();
        match components.len() {
            // @TODO extra, utf8 percent decode
            3 => Ok(ResourceRef {
                resource: parse_component(components[0])?,
                type_name: parse_component(components[1])?,
                id: parse_component(components[2])?,
                extra: vec![],
            }),
            4 => Ok(ResourceRef {
                resource: parse_component(components[0])?,
                type_name: parse_component(components[1])?,
                id: parse_component(components[2])?,
                extra: vec![],
            }),
            i => Err(ParseResourceErr::InvalidLength(i)),
        }
    }
}
fn parse_component(s: &str) -> Result<String, ParseResourceErr> {
    Ok(percent_decode(s.as_bytes()).decode_utf8().map_err(|_| ParseResourceErr::DecodeErr)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_extra() {
        // @TODO helper for this?
        let r = ResourceRef::without_extra("catalog", "movie", "top/.f");
        assert_eq!(r, ResourceRef::from_str(&r.to_string()).unwrap());
    }

    #[test]
    fn with_extra() {
    }
}

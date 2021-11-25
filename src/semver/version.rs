// https://github.com/dtolnay/semver/blob/master/src/lib.rs

use std::cmp::Ordering;
use std::convert::TryInto;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref VERSION_PATTERN: Regex = Regex::new(r#"(?P<major>\d+).(?P<minor>\d+).(?P<patch>\d+)"#).unwrap();
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    // pub pre: Prerelease,
    // pub build: BuildMetadata,
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let major_order = self.major.cmp(&other.major);
        if major_order.is_ne() {
            return Some(major_order);
        }
        let minor_order = self.minor.cmp(&other.minor);
        if minor_order.is_ne() {
            return Some(minor_order);
        }
        let patch_order = self.patch.cmp(&other.patch);
        if patch_order.is_ne() {
            return Some(patch_order);
        }
        None
    }
}

impl TryInto<Version> for String {
    type Error = String;

    fn try_into(self) -> Result<Version, Self::Error> {
        if let Some(captures) = VERSION_PATTERN.captures(&self) {
            let major = captures.name("major").expect("cannot find major version").as_str();
            let minor = captures.name("minor").expect("cannot find minor version").as_str();
            let patch = captures.name("patch").expect("cannot find patch version").as_str();
            Ok(Version {
                major: major.parse().expect(&format!("major {} is not valid", major)),
                minor: minor.parse().expect(&format!("minor {} is not valid", minor)),
                patch: patch.parse().expect(&format!("patch {} is not valid", patch)),
            })
        } else {
            Err(String::from("pattern not matched"))
        }
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use crate::semver::version::Version;

    #[test]
    fn parse() {
        let version1: Version = String::from("1.2.3").try_into().unwrap();
        dbg!(&version1);
        let version2: Version = String::from("1.2.4").try_into().unwrap();
        dbg!(&version2);
        assert!(version1.cmp(&version2).is_lt());
        assert!(version2.cmp(&version1).is_gt());
    }
}
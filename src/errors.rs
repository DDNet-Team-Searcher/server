use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Error<'a> {
    status: &'a str,
}

pub enum Errors {
    AccessDenied,
    BadData,
    Unknown,
}

impl Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Self::BadData => Error { status: "BAD_DATA" },
            Self::Unknown => Error {
                status: "SOMETHING_HAPPENNED_IDK_MYSELF",
            },
            Self::AccessDenied => Error {
                status: "ACCESS_DENIED",
            },
        };

        write!(f, "{}", serde_json::to_string(&res).unwrap()).unwrap();

        Ok(())
    }
}

impl Debug for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string()).unwrap();

        Ok(())
    }
}

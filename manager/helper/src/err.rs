use std::fmt;

#[derive(Debug)]
pub struct IndividualErrorList {
    list: Vec<anyhow::Error>,
}

impl fmt::Display for IndividualErrorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.list)
    }
}

impl From<Vec<anyhow::Error>> for IndividualErrorList {
    fn from(list: Vec<anyhow::Error>) -> Self { IndividualErrorList { list } }
}

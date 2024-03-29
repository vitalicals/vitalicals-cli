use bdk::bitcoin::OutPoint;

/// The id of vitalicals 's Resource
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub struct LocationRef(OutPoint);

impl ToString for LocationRef {
    fn to_string(&self) -> String {
        format!("{}i{}", self.0.txid, self.0.vout)
    }
}

/// The id of vitalicals 's Resource
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Id {
    Location(LocationRef),
    Number(u64),
}

impl Default for Id {
    fn default() -> Self {
        Self::Location(LocationRef::default())
    }
}

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Location(point) => write!(f, "Id({})", point.to_string()),
            Self::Number(n) => write!(f, "Id({})", n),
        }
    }
}

impl ToString for Id {
    fn to_string(&self) -> String {
        match self {
            Self::Location(point) => point.to_string(),
            Self::Number(number) => number.to_string(),
        }
    }
}

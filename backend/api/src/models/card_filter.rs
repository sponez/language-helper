use serde::{Deserialize, Serialize};

use crate::models::card::CardType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CardFilter {
    All,
    Straight,
    Reverse,
}

impl CardFilter {
    pub fn matches(self, card_type: &CardType) -> bool {
        match self {
            Self::All => true,
            Self::Straight => matches!(card_type, CardType::Straight),
            Self::Reverse => matches!(card_type, CardType::Reverse),
        }
    }
}

impl std::fmt::Display for CardFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "All"),
            Self::Straight => write!(f, "Straight"),
            Self::Reverse => write!(f, "Reverse"),
        }
    }
}

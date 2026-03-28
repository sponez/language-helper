use super::card::CardType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

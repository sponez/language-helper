/// Messages that can be sent within the cards menu router
#[derive(Debug, Clone)]
pub enum Message {
    /// Manage Cards button pressed
    ManageCards,
    /// Learn button pressed
    Learn,
    /// Test button pressed
    Test,
    /// Repeat button pressed
    Repeat,
    /// Back button pressed
    Back,
}

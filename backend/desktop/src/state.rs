use std::sync::Arc;

use application::ports::input::local_user::LocalUserUsecase;
use lh_bootstrap::BootstrapBridge;

pub struct DesktopState {
    local_users: Arc<dyn LocalUserUsecase>,
}

impl DesktopState {
    pub fn new(bridge: BootstrapBridge) -> Self {
        Self {
            local_users: bridge.local_users(),
        }
    }

    pub fn local_users(&self) -> Arc<dyn LocalUserUsecase> {
        Arc::clone(&self.local_users)
    }
}

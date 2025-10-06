use crate::apis::user_api::UsersApi;

 pub trait AppApi: Send + Sync {
    fn users_api(&self) -> &dyn UsersApi;
 }
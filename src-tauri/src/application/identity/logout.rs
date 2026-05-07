use crate::application::errors::AppResult;
use crate::application::identity::service::IdentityService;

impl IdentityService {
    /// 登出：清空 SessionStore（PROMPT 第 86 行 内存 Session）。
    /// 不写审计——session 内存清掉就是登出，没什么需要追溯的；登入登出已经
    /// 由 LoginSucceeded / LoginFailed / SessionForceLogout 覆盖。
    pub fn logout(&self) -> AppResult<()> {
        self.session_store.clear();
        Ok(())
    }
}

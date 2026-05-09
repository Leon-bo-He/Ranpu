use crate::application::ports::errors::RepositoryError;
use crate::domain::cart::cart::Cart;
use crate::domain::shared::id::WorkspaceId;

pub trait CartRepository: Send + Sync {
    /// 取指定工作区的购物车. 不存在返回空 Cart 而非 None.
    fn load(&self, workspace_id: WorkspaceId) -> Result<Cart, RepositoryError>;

    /// 整体覆盖保存：把 Cart 当前 items 写入 cart_items 表（DELETE + INSERT
    /// 单事务）。一次性提交保证不变量。
    fn save(&self, cart: &Cart) -> Result<(), RepositoryError>;
}

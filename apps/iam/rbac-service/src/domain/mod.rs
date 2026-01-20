pub mod entities;
pub mod repositories;

pub use entities::permission::Permission;
pub use entities::role::Role;
pub use repositories::{PermissionRepository, RoleRepository};

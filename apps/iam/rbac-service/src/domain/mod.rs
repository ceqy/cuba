pub mod entities;
pub mod repositories;

pub use entities::role::Role;
pub use entities::permission::Permission;
pub use repositories::{RoleRepository, PermissionRepository};

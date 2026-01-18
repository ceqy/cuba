//! Mapper 模块 - 统一处理 Proto ↔ DTO ↔ Domain 的转换
//!
//! 解决问题：
//! - 消除 400+ 行重复映射代码
//! - 统一错误处理模式
//! - 集中验证逻辑
//! - 提高可测试性

pub mod timestamp_mapper;
pub mod string_mapper;
pub mod payment_mapper;
pub mod line_item_mapper;

pub use timestamp_mapper::*;
pub use string_mapper::*;
pub use payment_mapper::*;
pub use line_item_mapper::*;

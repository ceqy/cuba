//! gRPC 模块 - 包含 proto 生成代码

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod fi {
    pub mod gl {
        pub mod v1 {
            tonic::include_proto!("fi.gl.v1");

            pub const FILE_DESCRIPTOR_SET: &[u8] =
                tonic::include_file_descriptor_set!("gl_descriptor");
        }
    }
}

// Re-export for easier access
pub use fi::gl::v1::*;

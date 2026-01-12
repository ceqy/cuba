// Include generated proto code

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod iam {
    pub mod v1 {
        pub mod common {
            tonic::include_proto!("iam.v1.common");
        }
        pub mod auth {
            tonic::include_proto!("iam.v1.auth");
        }
        pub mod rbac {
            tonic::include_proto!("iam.v1.rbac");
        }
        pub mod oauth {
            tonic::include_proto!("iam.v1.oauth");
        }

        pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("iam_descriptor");
    }
}

// Re-export common types for easier access if necessary
// pub use iam::v1::common::*;

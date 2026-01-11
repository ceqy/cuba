// Include generated proto code

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod iam {
    pub mod v1 {
        tonic::include_proto!("iam.v1");
        pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("iam_descriptor");
    }
}

pub use iam::v1::*;

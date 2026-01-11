pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod cs {
    pub mod wc {
        pub mod v1 {
            tonic::include_proto!("cs.wc.v1");
            pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor");
        }
    }
}

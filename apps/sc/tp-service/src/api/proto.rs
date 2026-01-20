pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}
pub mod sc {
    pub mod tp {
        pub mod v1 {
            tonic::include_proto!("sc.tp.v1");
            pub const FILE_DESCRIPTOR_SET: &[u8] =
                tonic::include_file_descriptor_set!("descriptor");
        }
    }
}

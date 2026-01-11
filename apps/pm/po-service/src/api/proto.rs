pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod pm {
    pub mod po {
        pub mod v1 {
            tonic::include_proto!("pm.po.v1");
            pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor");
        }
    }
}

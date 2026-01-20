pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod fi {
    pub mod ap {
        pub mod v1 {
            tonic::include_proto!("fi.ap.v1");
            pub const FILE_DESCRIPTOR_SET: &[u8] =
                tonic::include_file_descriptor_set!("descriptor");
        }
    }
    pub mod gl {
        pub mod v1 {
            tonic::include_proto!("fi.gl.v1");
        }
    }
}

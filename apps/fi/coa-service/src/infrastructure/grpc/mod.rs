//! gRPC module - contains proto generated code

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod fi {
    pub mod coa {
        pub mod v1 {
            tonic::include_proto!("fi.coa.v1");
            
            pub const FILE_DESCRIPTOR_SET: &[u8] = 
                tonic::include_file_descriptor_set!("coa_descriptor");
        }
    }
}

// Re-export for easier access
pub use fi::coa::v1::*;

//! AR/AP Service - Proto Module
//!
//! 引入生成的 Proto 代码

pub mod finance {
    pub mod arap {
        tonic::include_proto!("finance.arap");
        
        pub const FILE_DESCRIPTOR_SET: &[u8] = 
            tonic::include_file_descriptor_set!("arap_service_descriptor");
    }
}

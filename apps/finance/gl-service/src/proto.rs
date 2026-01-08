// 引入生成的 proto 代码
pub mod common {
    tonic::include_proto!("common");
}

pub mod finance {
    pub mod gl {
        tonic::include_proto!("finance.gl");
        
        // 导出文件描述符集以便用于反射
        pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("gl_service_descriptor");
    }
}

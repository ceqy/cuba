// 引入生成的 proto 代码
pub mod enterprise {
    pub mod common {
        tonic::include_proto!("enterprise.common");
    }
    pub mod finance {
        pub mod gl {
            tonic::include_proto!("enterprise.finance.gl");
            
            // 导出文件描述符集以便用于反射
            pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("finance_descriptor");
        }
    }
}

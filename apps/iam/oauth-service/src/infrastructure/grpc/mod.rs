pub mod iam {
    pub mod oauth {
        pub mod v1 {
            tonic::include_proto!("iam.oauth.v1");
        }
    }
}
pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

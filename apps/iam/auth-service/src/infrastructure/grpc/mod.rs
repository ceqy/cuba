pub mod iam {
    pub mod auth {
        pub mod v1 {
            tonic::include_proto!("iam.auth.v1");
        }
    }
    pub mod rbac {
        pub mod v1 {
            tonic::include_proto!("iam.rbac.v1");
        }
    }
}
pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

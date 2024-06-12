pub mod sited_io {
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("./FILE_DESCRIPTOR_SET");

    pub mod websites {
        pub mod v1 {
            include!("sited_io.websites.v1.rs");
        }
    }

    pub mod pagination {
        pub mod v1 {
            include!("sited_io.pagination.v1.rs");
        }
    }
}

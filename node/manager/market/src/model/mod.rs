mod database;
pub use database::{AuctionStatus, BidRecord, NodeRecord};

mod disk;
pub use disk::NodeRecordDisk;

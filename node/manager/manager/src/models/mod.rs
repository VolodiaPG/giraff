use uuid::Uuid;
pub type BidId = Uuid;
pub type NodeId = Uuid;

mod outputs;
pub use outputs::{Bid, Satisfiable};

mod database;
pub use database::{BidRecord, ProvisionedRecord};

mod inputs;

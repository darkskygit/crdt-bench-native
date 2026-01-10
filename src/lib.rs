pub use crate::criterion::{automerge_parallel, entry};

mod automerge;
mod crdt;
mod criterion;
mod doc_size;

pub use crdt::{
    automerge::AutomergeDoc, diamond_types::DiamondTypeDoc, loro::Loro, merge, y_octo::YOctoDoc,
    yrs::YrsDoc, Crdt,
};

pub use doc_size::run_doc_size;

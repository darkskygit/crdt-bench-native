pub use crate::automerge::{get_automerge_actions, TextAction};
pub use crate::criterion::{
    automerge_parallel, entry, list_random_insert_positions, parallel_automerge_batches,
    run_text_append, run_text_backspace, run_text_random_edits, run_text_tombstone,
};

mod automerge;
mod crdt;
mod criterion;
mod doc_size;

pub use crdt::{
    automerge::AutomergeDoc, diamond_types::DiamondTypeDoc, loro::Loro, merge, y_octo::YOctoDoc,
    yrs::YrsDoc, Crdt,
};

pub use doc_size::run_doc_size;

//! ULID generator adapter.

use ulid::Ulid;

use crate::app::port::IdGenerator;

/// Generates sortable ULIDs.
#[derive(Debug, Clone, Copy, Default)]
pub struct UlidGenerator;

impl IdGenerator for UlidGenerator {
    fn new_id(&self) -> String {
        Ulid::new().to_string()
    }
}

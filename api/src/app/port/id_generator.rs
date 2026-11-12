//! Identifier generation.

/// Creates stable, sortable identifiers. Use cases call this instead of
/// importing a UUID/ULID library directly.
pub trait IdGenerator: Send + Sync {
    /// A new identifier.
    fn new_id(&self) -> String;
}

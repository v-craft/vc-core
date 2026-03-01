use bitflags::bitflags;

bitflags! {
    /// Bitflags representing system states and requirements.
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SystemFlags: u8 {
        /// Set if system cannot be sent across threads
        const MAIN_THREAD       = 1 << 0;
        /// Set if system requires exclusive World access
        const EXCLUSIVE      = 1 << 1;
    }
}

pub type Instant<const NOM: u32> = fugit::Instant<u64, NOM, 1>;
pub type InstantSecs = Instant<1>;

/// Clock that fetches `Instant`s relative to the unix epoch.
pub trait RealTimeClock {
    fn get_time(&self) -> InstantSecs;
}

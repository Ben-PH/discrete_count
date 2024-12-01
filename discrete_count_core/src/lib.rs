#![no_std]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

#[allow(type_alias_bounds)]
pub type CountRaw<C: Counter> = <<C as Counter>::Reader as CountReader>::RawData;
/// Encapsulates the types and operations assosciated with a count-oriented driver.
pub trait Counter {
    type Reader: CountReader;
    /// Encapsulates the magnitude of what is being measured for each count.
    ///
    /// E.g.:
    /// - `(typenum::U1, typenum::U10000)` for a 10khz oscilator.
    /// - a 1024 PPR AB encoder has tau/4096 resolution, ~= 1/652 radians:
    ///   `(typenum::U1, typenum::U652)`
    type Resolution;

    /// Encapsulates what is being measured. Some possible measures:
    ///
    /// - Degrees
    /// - arc-seconds
    /// - Radians
    /// - Milliseconds
    /// - Meters
    ///
    /// E.g.
    /// ```rust ignore
    ///
    /// // Basic, short-life 1khz resolution time-measure
    /// struct MilliSeconds(u32);
    ///
    /// // a 1024ppr encoder has log_2(1024 * 4) = 12 bits of information.
    /// // Fixing the fractional measure to be 12 bits encapsulates this precision.
    /// // A typical valve rarely makes a full rotation, so the smaller bit-count
    /// // does not matter, and 16 bits is the smallest that can hold 12 fractional bits.
    /// //
    /// // We use Tau in its name, because we are thinking in terms of full-rotations,
    /// // and `Tau` radians is one full rotation
    /// struct ValveTau(discrete_count::re_exports::fixed::types::I6F12);
    ///
    /// // Advanced manufacturing equipment might need rotary stages that need to track
    /// // up to 2^16 rotations, with pricesion up to 2^-16 radians.
    /// struct RotaryStageTau(discrete_count::re_exports::fixed::types::I16F16);
    ///
    /// // Encapsulate 32-bit POSIX timestamps
    /// struct Seconds(u32);
    ///
    /// // Encapsulate 64-bit POSIX timestamps
    /// struct BigSeconds(u64);
    /// ```
    type Measure;

    fn update_count_state(
        &mut self,
        count: CountRaw<Self>,
    ) -> Result<(), <Self::Reader as CountReader>::ReadErr>;
    fn read_count_state(&self) -> &CountRaw<Self>;
    /// Updates the internal count state.
    /// # Errors
    ///
    /// If the internal count-read errors out, the implementation should return an error,
    /// and not update the count.
    fn try_update_count(&mut self) -> Result<(), <Self::Reader as CountReader>::ReadErr>;
    /// Reads the count, applying scale to read measure.
    /// # Errors
    ///
    /// If the internal count-read errors out, the implementation should return an error.
    fn try_read_measure(&self) -> Result<Self::Measure, <Self::Reader as CountReader>::ReadErr>;
    fn measure_count_state(&self) -> Self::Measure;
    /// Derive the magnitude being measured. E.g.
    /// ```rust ignore
    /// let stage_rotation = Count::measure(&stage, stage.try_read_measure()?);
    /// let rotation_count = stage_rotation.int();
    /// let rotation_angle = stage_rotation.frac();
    /// ```
    fn try_update_and_measure(
        &mut self,
        count: &CountRaw<Self>,
    ) -> Result<Self::Measure, <Self::Reader as CountReader>::ReadErr>;
    fn direct_measure() -> Result<Self::Measure, <Self::Reader as CountReader>::ReadErr> {
        Ok(Self::measure_count(&<Self::Reader as CountReader>::read()?))
    }
    fn measure_count(count: &CountRaw<Self>) -> Self::Measure;
}

/// Used to interface directly with the source of a count.
/// Typical implementation is to read directly from the relevant hardware register.
pub trait CountReader {
    type ReadErr;
    type RawData;
    fn read() -> Result<Self::RawData, Self::ReadErr>;
}

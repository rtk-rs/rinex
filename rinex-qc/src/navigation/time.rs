use gnss_rtk::prelude::{AbsoluteTime, Epoch, TimeScale};

/// [AbsoluteTimeContext] implements the solver [AbsoluteTime] interface
/// with the timescale conversions provided by hifitime.
pub struct AbsoluteTimeContext {}

impl AbsoluteTime for AbsoluteTimeContext {
    fn new_epoch(&mut self, _: Epoch) {}

    fn epoch_correction(&self, t: Epoch, target: TimeScale) -> Epoch {
        t.to_time_scale(target)
    }
}

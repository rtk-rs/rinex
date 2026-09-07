use crate::context::QcContext;

use gnss_rtk::prelude::{
    BiasRuntime, EnvironmentalBias, IonosphereModel, KbModel, TroposphereModel,
};

/// [EnvironmentContext] implements the solver [EnvironmentalBias] interface
/// from the models available in the [QcContext].
pub struct EnvironmentContext {
    /// Klobuchar model published in the navigation data, when available
    kb_model: Option<KbModel>,
}

impl EnvironmentContext {
    /// Height of the Klobuchar ionosphere layer, in kilometers
    const KB_LAYER_HEIGHT_KM: f64 = 350.0;

    /// Builds the [EnvironmentContext] of this [QcContext]
    pub fn new(ctx: &QcContext) -> Self {
        let kb_model = ctx.nav_dataset.as_ref().and_then(|nav| {
            nav.header
                .ionod_corrections
                .values()
                .find_map(|model| model.as_klobuchar())
                .map(|kb| KbModel {
                    alpha: kb.alpha,
                    beta: kb.beta,
                    h_km: Self::KB_LAYER_HEIGHT_KM,
                })
        });

        Self { kb_model }
    }
}

impl EnvironmentalBias for EnvironmentContext {
    fn troposphere_bias_m(&self, rtm: &BiasRuntime) -> f64 {
        TroposphereModel::Niel.bias_m(rtm)
    }

    fn ionosphere_bias_m(&self, rtm: &BiasRuntime) -> f64 {
        match self.kb_model {
            Some(kb_model) => IonosphereModel::KbModel(kb_model).bias_m(rtm),
            None => 0.0,
        }
    }
}

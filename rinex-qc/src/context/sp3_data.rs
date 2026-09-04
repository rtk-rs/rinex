use crate::{
    context::{meta::MetaData, QcContext},
    QcCtxError,
};

use sp3::prelude::SP3;

use qc_traits::Merge;

impl QcContext {
    /// Load a single [SP3] into [QcContext].
    /// File revision must be supported, file must be correctly formatted
    /// for this operation to be effective.
    pub fn load_sp3(&mut self, meta: &mut MetaData, sp3: SP3) -> Result<(), QcCtxError> {
        // SP3 classification is always feasible
        meta.unique_id = Some(sp3.header.agency.to_string());

        // store correctly
        if let Some(inner) = self.sp3_dataset.get_mut(&meta) {
            inner.merge_mut(&sp3)?;
        } else {
            self.sp3_dataset.insert(meta.clone(), sp3);
        }

        Ok(())
    }

    /// Returns true if this [QcContext] contains at least
    /// one Precise Orbit [SP3] product
    pub fn has_precise_orbits(&self) -> bool {
        !self.sp3_dataset.is_empty()
    }
}

#[cfg(test)]
mod test {

    use crate::{
        cfg::QcConfig,
        context::{meta::MetaData, QcContext},
    };

    #[test]
    #[cfg(feature = "flate2")]
    fn sp3_indexing() {
        let path = format!(
            "{}/../test_resources/SP3/COD0MGXFIN_20230500000_01D_05M_ORB.SP3.gz",
            env!("CARGO_MANIFEST_DIR")
        );

        let cfg = QcConfig::default();
        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_gzip_file(&path).unwrap();
        assert!(ctx.has_precise_orbits());
        assert_eq!(ctx.sp3_dataset.len(), 1);

        ctx.load_file(&path).unwrap();
        assert!(ctx.has_precise_orbits());
        assert_eq!(ctx.sp3_dataset.len(), 1);

        let meta = MetaData {
            name: "COD0MGXFIN_20230500000_01D_05M_ORB".to_string(),
            extension: "SP3.gz".to_string(),
            unique_id: Some("AIUB".to_string()),
        };

        assert!(ctx.sp3_dataset.get(&meta).is_some());
    }
}

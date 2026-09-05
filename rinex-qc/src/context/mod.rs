use std::{
    collections::HashMap,
    env,
    fs::{create_dir_all, rename, File},
    io::Write,
    path::Path,
    sync::Mutex,
};

use rinex::prelude::{nav::Almanac, Rinex};

use anise::{
    almanac::metaload::{MetaAlmanac, MetaFile},
    constants::frames::{EARTH_ITRF93, IAU_EARTH_FRAME},
    prelude::Frame,
};

use qc_traits::{Filter, Preprocessing, Repair, RepairTrait};

// Context post processing.
// This that can only be achieved by stacking more than one RINEX
// and possibly one SP3.
mod processing;

pub(crate) mod ionex;
pub(crate) mod meta;
pub(crate) mod meteo;
pub(crate) mod nav;
pub(crate) mod obs;
pub(crate) mod rnx;
pub(crate) mod session;
pub(crate) mod tropo;

#[cfg(feature = "sp3")]
#[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
pub(crate) mod sp3_data;

#[cfg(feature = "sp3")]
use sp3::prelude::SP3;

use crate::{
    analysis::QcAnalysis,
    cfg::{QcConfig, QcFrameModel},
    context::meta::{MetaData, ObsMetaData},
    QcCtxError,
};

/// [QcContext] is a general structure capable to store most common
/// GNSS data. It is dedicated to post processing workflows,
/// precise timing or atmosphere analysis.
pub struct QcContext {
    /// [QcConfig] used to deploy this [QcContext]
    pub cfg: QcConfig,

    /// Latest Almanac to use during this session.
    pub almanac: Almanac,

    /// ECEF frame to use during this session. Based off [Almanac].
    pub earth_cef: Frame,

    /// Observations [Rinex] stored by [MetaData]
    pub obs_dataset: HashMap<ObsMetaData, Rinex>,

    /// Possible Navigation [Rinex]
    pub nav_dataset: Option<Rinex>,

    /// Possible IONEx [Rinex]
    pub ionex_dataset: Option<Rinex>,

    /// Meteo [Rinex] stored by [MetaData]
    pub meteo_dataset: HashMap<MetaData, Rinex>,

    /// Possible [SP3] fileset
    #[cfg(feature = "sp3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
    pub sp3_dataset: HashMap<MetaData, SP3>,
}

/// [Almanac] shared by every [QcContext] of this process
static SHARED_ALMANAC: Mutex<Option<Almanac>> = Mutex::new(None);

impl QcContext {
    /// ANISE storage location
    const ANISE_ALMANAC_STORAGE: &str = ".cache";

    /// Returns [MetaFile] for anise DE440s.bsp
    fn nyx_anise_de440s_bsp() -> MetaFile {
        MetaFile {
            crc32: Some(1921414410),
            uri: String::from("http://public-data.nyxspace.com/anise/de440s.bsp"),
        }
    }

    /// Returns [MetaFile] for anise PCK11.pca
    fn nyx_anise_pck11_pca() -> MetaFile {
        MetaFile {
            crc32: Some(0x8213b6e9),
            uri: String::from("http://public-data.nyxspace.com/anise/v0.4/pck11.pca"),
        }
    }

    /// Returns [MetaFile] for daily JPL high precision bpc
    fn nyx_anise_jpl_bpc() -> MetaFile {
        MetaFile {
            crc32: None,
            uri:
                "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc"
                    .to_string(),
        }
    }

    /// Returns the [Almanac] shared by every [QcContext] of this process.
    /// The first call downloads (or retrieves from local storage) the
    /// almanac files, the following calls return a copy.
    fn shared_almanac() -> Result<Almanac, QcCtxError> {
        let mut shared = SHARED_ALMANAC.lock().unwrap_or_else(|e| e.into_inner());

        if let Some(almanac) = shared.as_ref() {
            return Ok(almanac.clone());
        }

        let almanac = Self::load_almanac()?;
        *shared = Some(almanac.clone());
        Ok(almanac)
    }

    /// Method to either download, retrieve or create a basic [Almanac].
    /// This will try to download the highest JPL model, and requires
    /// internet access once a day.
    ///
    /// Processes setting up the almanac at the same time (tests for example)
    /// take turns: the download and the local storage are protected by an
    /// exclusive file lock.
    fn load_almanac() -> Result<Almanac, QcCtxError> {
        let storage = Path::new(env!("CARGO_MANIFEST_DIR")).join(Self::ANISE_ALMANAC_STORAGE);

        create_dir_all(&storage).map_err(|_| QcCtxError::IO)?;

        // released when dropped, at the end of this function
        let lock = File::create(storage.join("anise.lock")).map_err(|_| QcCtxError::IO)?;
        lock.lock().map_err(|_| QcCtxError::IO)?;

        let local_storage = storage.join("anise.dhall");
        let local_storage_s = local_storage.to_string_lossy().to_string();

        let mut initial_setup = false;

        // Meta almanac for local storage management
        let mut meta_almanac = match MetaAlmanac::new(local_storage_s) {
            Ok(meta) => {
                debug!("(anise) from local storage");
                meta
            },
            Err(_) => {
                debug!("(anise) local storage setup");
                initial_setup = true;
                MetaAlmanac {
                    files: vec![
                        Self::nyx_anise_de440s_bsp(),
                        Self::nyx_anise_pck11_pca(),
                        Self::nyx_anise_jpl_bpc(),
                    ],
                }
            },
        };

        // download (if need be)
        let almanac = meta_almanac.process(true)?;

        if initial_setup {
            let updated = meta_almanac.dumps()?;

            // written aside then renamed: readers never see a partial file
            let tmp_storage = storage.join("anise.dhall.tmp");

            File::create(&tmp_storage)
                .and_then(|mut fd| fd.write_all(updated.as_bytes()))
                .and_then(|_| rename(&tmp_storage, &local_storage))
                .map_err(|_| QcCtxError::IO)?;
        }

        Ok(almanac)
    }

    /// Returns the reference [Frame] to work with: the highest precision
    /// model available in the [Almanac] among the prefered ones.
    fn earth_frame(almanac: &Almanac, prefered: QcFrameModel) -> Result<Frame, QcCtxError> {
        if prefered == QcFrameModel::ITRF93 {
            // try to form the EARTH ITRF93 frame model
            match almanac.frame_from_uid(EARTH_ITRF93) {
                Ok(itrf93) => {
                    info!("earth_itrf93 frame model loaded");
                    return Ok(itrf93);
                },
                Err(e) => {
                    error!("(anise) itrf93: {}", e);
                },
            }
        }

        let earth_cef = almanac.frame_from_uid(IAU_EARTH_FRAME)?;
        warn!("deployed with offline model");
        Ok(earth_cef)
    }

    /// Method to either download, retrieve or create
    /// a basic [Almanac] and reference [Frame] to work with.
    fn build_almanac_frame_model(prefered: QcFrameModel) -> Result<(Almanac, Frame), QcCtxError> {
        let almanac = Self::shared_almanac()?;
        let earth_cef = Self::earth_frame(&almanac, prefered)?;
        Ok((almanac, earth_cef))
    }

    /// Creates a new [QcContext] with [QcConfig] configuration preset.
    pub fn new(cfg: QcConfig) -> Result<Self, QcCtxError> {
        let mut cfg = cfg.clone();

        let (almanac, earth_cef) = Self::build_almanac_frame_model(cfg.navi.frame_model)?;

        if earth_cef == EARTH_ITRF93 {
            cfg.navi.frame_model = QcFrameModel::ITRF93;
        } else {
            cfg.navi.frame_model = QcFrameModel::IAU;
        }

        let s = Self {
            cfg,
            almanac,
            earth_cef,
            obs_dataset: Default::default(),
            nav_dataset: Default::default(),
            meteo_dataset: Default::default(),
            ionex_dataset: Default::default(),
            #[cfg(feature = "sp3")]
            sp3_dataset: Default::default(),
        };

        s.deploy()?;
        Ok(s)
    }

    /// Smart data loader, that will automatically pick up the provided
    /// format (if supported) and load it into the [QcContext].
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), QcCtxError> {
        let path = path.as_ref();

        #[cfg(feature = "flate2")]
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
        {
            return self.load_gzip_file(path);
        }

        let mut meta = MetaData::new(path)?;

        if let Ok(rinex) = Rinex::from_file(path) {
            self.load_rinex(&mut meta, rinex)?;
            info!(
                "{} (RINEx) loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            return Ok(());
        }

        #[cfg(feature = "sp3")]
        if let Ok(sp3) = SP3::from_file(path) {
            self.load_sp3(&mut meta, sp3)?;
            info!(
                "{} (SP3) loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            return Ok(());
        }

        Err(QcCtxError::NonSupportedFormat)
    }

    /// Smart data loader, that will automatically pick up the provided
    /// format (if supported) and load it into the [QcContext].
    #[cfg(feature = "flate2")]
    pub fn load_gzip_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), QcCtxError> {
        let path = path.as_ref();
        let mut meta = MetaData::new(path)?;

        if let Ok(rinex) = Rinex::from_gzip_file(path) {
            self.load_rinex(&mut meta, rinex)?;
            info!(
                "{} (RINEx) loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            return Ok(());
        }

        #[cfg(feature = "sp3")]
        if let Ok(sp3) = SP3::from_gzip_file(path) {
            self.load_sp3(&mut meta, sp3)?;
            info!(
                "{} (SP3) loaded",
                path.file_stem().unwrap_or_default().to_string_lossy()
            );
            return Ok(());
        }

        Err(QcCtxError::NonSupportedFormat)
    }

    /// Applies [Filter] operation to this [QcContext]
    pub fn filter_mut(&mut self, filter: &Filter) {
        for (_, rinex) in self.obs_dataset.iter_mut() {
            rinex.filter_mut(&filter);
        }
        if let Some(rinex) = &mut self.nav_dataset {
            rinex.filter_mut(&filter);
        }
        if let Some(rinex) = &mut self.ionex_dataset {
            rinex.filter_mut(&filter);
        }
        for (_, rinex) in self.meteo_dataset.iter_mut() {
            rinex.filter_mut(&filter);
        }
    }

    /// Applies [Repair] operation to this [QcContext].
    /// This may only apply to Observation and Navigation datasets.
    pub fn repair_mut(&mut self, repair: Repair) {
        for (_, rinex) in self.obs_dataset.iter_mut() {
            rinex.repair_mut(repair);
        }

        if let Some(rinex) = &mut self.nav_dataset {
            rinex.repair_mut(repair);
        }
    }

    /// Analyze complete dataset
    pub fn analyze(&self) -> QcAnalysis {
        QcAnalysis::new(self)
    }
}

impl std::fmt::Debug for QcContext {
    /// Debug formatting, prints all loaded files per Product category.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, _) in &self.obs_dataset {
            write!(f, "OBS RINEx: {}", k.meta.name)?;
        }

        for (k, _) in &self.meteo_dataset {
            write!(f, "Meteo RINEx: {}", k.name)?;
        }

        #[cfg(feature = "sp3")]
        for (k, _) in &self.sp3_dataset {
            if let Some(unique_id) = &k.unique_id {
                write!(f, "({}) SP3: {}", unique_id, k.name)?;
            } else {
                write!(f, "SP3: {}", k.name)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{cfg::QcConfig, context::QcContext};
    use std::thread;

    /// Contexts deployed concurrently share one almanac setup
    #[test]
    fn concurrent_deployment() {
        let handles = (0..8)
            .map(|_| thread::spawn(|| QcContext::new(QcConfig::default()).map(|_| ())))
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap().expect("context deployment failure");
        }

        let storage =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(QcContext::ANISE_ALMANAC_STORAGE);

        assert!(storage.join("anise.dhall").is_file());
        assert!(storage.join("anise.lock").is_file());
        assert!(!storage.join("anise.dhall.tmp").exists());
    }
}

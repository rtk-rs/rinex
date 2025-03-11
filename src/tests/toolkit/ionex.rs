use std::str::FromStr;

use crate::{
    ionex::{IonexKey, MappingFunction, Quantized, QuantizedCoordinates},
    prelude::{Epoch, Header, Rinex, RinexType},
    tests::toolkit::{generic_rinex_test, TimeFrame},
};

pub struct TecPoint<'a> {
    pub t: &'a str,
    pub lat_ddeg: f64,
    pub lat_exponent: i8,
    pub long_ddeg: f64,
    pub long_exponent: i8,
    pub alt_km: f64,
    pub alt_exponent: i8,
    pub tecu: f64,
    pub rms: Option<f64>,
}

/// Basic tests for Observation [Rinex]
fn basic_header_tests(
    dut: &Header,
    epoch_of_first_map: &str,
    epoch_of_last_map: &str,
    nb_maps: usize,
    lat_grid_start: f64,
    lat_grid_end: f64,
    lat_grid_spacing: f64,
    long_grid_start: f64,
    long_grid_end: f64,
    long_grid_spacing: f64,
    h_grid_start: Option<f64>,
    h_grid_end: Option<f64>,
    h_grid_spacing: Option<f64>,
    exponent: i8,
    base_radius: f32,
    elevation_cutoff: f32,
    mapping_function: Option<&str>,
) {
    assert!(dut.obs.is_none());
    assert!(dut.meteo.is_none());
    assert!(dut.clock.is_none());
    assert!(dut.ionex.is_some());

    let specs = dut.ionex.as_ref().expect("missing specific specs");

    assert_eq!(specs.number_of_maps, nb_maps);

    let epoch_of_first_map = Epoch::from_str(epoch_of_first_map.trim()).unwrap();
    assert_eq!(specs.epoch_of_first_map, epoch_of_first_map);

    let epoch_of_last_map = Epoch::from_str(epoch_of_last_map.trim()).unwrap();
    assert_eq!(specs.epoch_of_last_map, epoch_of_last_map);

    assert_eq!(specs.grid.latitude.start, lat_grid_start);
    assert_eq!(specs.grid.latitude.end, lat_grid_end);
    assert_eq!(specs.grid.latitude.spacing, lat_grid_spacing);

    assert_eq!(specs.grid.longitude.start, long_grid_start);
    assert_eq!(specs.grid.longitude.end, long_grid_end);
    assert_eq!(specs.grid.longitude.spacing, long_grid_spacing);

    if let Some(h_grid_start) = h_grid_start {
        assert_eq!(specs.grid.height.start, h_grid_start);
    }
    if let Some(h_grid_end) = h_grid_end {
        assert_eq!(specs.grid.height.end, h_grid_end);
    }
    if let Some(h_grid_spacing) = h_grid_spacing {
        assert_eq!(specs.grid.height.spacing, h_grid_spacing);
    }

    assert_eq!(specs.exponent, exponent);
    assert_eq!(specs.base_radius, base_radius);
    assert_eq!(specs.elevation_cutoff, elevation_cutoff);

    if let Some(mapping_func) = mapping_function {
        let mapping_func = MappingFunction::from_str(mapping_func).unwrap();
        assert_eq!(specs.mapping, Some(mapping_func));
    } else {
        assert!(specs.mapping.is_none());
    }
}

/// Generic test that we can use for Observation [Rinex]
pub fn generic_ionex_test(
    dut: &Rinex,
    version: &str,
    is_3d: bool,
    nb_maps: usize,
    epoch_of_first_map: &str,
    epoch_of_last_map: &str,
    lat_grid_start: f64,
    lat_grid_end: f64,
    lat_grid_spacing: f64,
    long_grid_start: f64,
    long_grid_end: f64,
    long_grid_spacing: f64,
    h_grid_start: Option<f64>,
    h_grid_end: Option<f64>,
    h_grid_spacing: Option<f64>,
    header_exponent: i8,
    base_radius: f32,
    elevation_cutoff: f32,
    mapping_function: Option<&str>,
    time_frame: TimeFrame,
    tec_points: Vec<TecPoint>,
) {
    assert!(dut.is_ionex());

    if is_3d {
        assert!(dut.is_ionex_3d());
    } else {
        assert!(dut.is_ionex_2d());
    }

    generic_rinex_test(
        dut,
        version,
        None,
        RinexType::IonosphereMaps,
        Some(time_frame),
    );

    basic_header_tests(
        &dut.header,
        epoch_of_first_map,
        epoch_of_last_map,
        nb_maps,
        lat_grid_start,
        lat_grid_end,
        lat_grid_spacing,
        long_grid_start,
        long_grid_end,
        long_grid_spacing,
        h_grid_start,
        h_grid_end,
        h_grid_spacing,
        header_exponent,
        base_radius,
        elevation_cutoff,
        mapping_function,
    );

    // Verify TEC in 3D (volume + isosurface)
    let dut_rec = dut.record.as_ionex().unwrap();

    for point in tec_points.iter() {
        let epoch = Epoch::from_str(point.t).unwrap();

        let lat_ddeg = point.lat_ddeg;
        let lat_exponent = Quantized::find_exponent(lat_grid_spacing);

        let long_ddeg = point.long_ddeg;
        let long_exponent = Quantized::find_exponent(long_grid_spacing);

        let alt_km = point.alt_km;
        let h_grid_spacing = h_grid_spacing.unwrap_or(0.0);
        let alt_exponent = Quantized::find_exponent(h_grid_spacing);

        let coordinates = QuantizedCoordinates::new(
            lat_ddeg,
            lat_exponent,
            long_ddeg,
            long_exponent,
            alt_km,
            alt_exponent,
        );

        let key = IonexKey { epoch, coordinates };

        let tec = dut_rec.get(&key).expect(&format!(
            "missing TEC for t={};lat={};long={};z={}",
            epoch, lat_ddeg, long_ddeg, alt_km
        ));

        let error = (tec.tecu() - point.tecu).abs();
        assert!(
            error < 1.0E-5,
            "bad tec value: {} versus {}",
            tec.tecu(),
            point.tecu
        );
    }
}

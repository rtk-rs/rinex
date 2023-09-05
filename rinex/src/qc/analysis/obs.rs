use crate::{carrier, observation::Snr, prelude::*, Carrier};
use itertools::Itertools;

use super::{pretty_array, QcOpts};
use std::collections::HashMap;

#[cfg(feature = "processing")]
use crate::preprocessing::*; // include preprocessing toolkit when feasible,
                             // for complete analysis

#[cfg(feature = "obs")]
use crate::observation::Observation; // having this feature unlocks full OBS RINEX analysis

/*
 * GNSS signal special formatting
 */
fn report_signals(list: &Vec<Carrier>) -> String {
    let mut s = String::with_capacity(3 * list.len());
    for index in 0..list.len() - 1 {
        s.push_str(&format!(
            "{} ({:.3} MHz), ",
            list[index],
            list[index].frequency_mhz()
        ));
    }
    s.push_str(&format!(
        "{} ({:.3} MHz)",
        list[list.len() - 1],
        list[list.len() - 1].frequency_mhz()
    ));
    s
}

/*
 * Epoch anomalies formatter
 */
fn report_anomalies<'a>(
    cs: &'a Vec<Epoch>,
    power: &'a Vec<Epoch>,
    other: &'a Vec<(Epoch, EpochFlag)>,
) -> Box<dyn RenderBox + 'a> {
    box_html! {
        table(class="table is-bordered") {
            thead {
                th {
                    : "Anomalies"
                }
            }
            tbody {
                tr {
                    th {
                        : "Power Failures"
                    }
                    @ if power.is_empty() {
                        td {
                            : "None"
                        }
                    } else {
                        td {
                            : format!("{:?}", power)
                        }
                        tr {
                            th {
                                : "Longest"
                            }
                            td {
                                //: power.iter().max_by(|(_, d1), (_, d2)| d1.cmp(d2)).unwrap().to_string()
                                : "TODO"
                            }
                            td {
                                : "Average Power failure"
                            }
                            td {
                                : "TODO"
                            }
                        }
                    }
                }
                tr {
                    th {
                        : "Cycle slip(s)"
                    }
                    @ if cs.is_empty() {
                        td {
                            : "None"
                        }
                    } else {
                        td {
                            : format!("{:?}", cs)
                        }
                    }
                }
                tr {
                    th {
                        : "Other anomalies"
                    }
                    @ if other.is_empty() {
                        td {
                            : "None"
                        }
                    } else {
                        td {
                            : "Epoch"
                        }
                        td {
                            : "Event"
                        }
                        @ for (e, event) in other {
                            td {
                                : ""
                            }
                            td {
                                : e.to_string()
                            }
                            //@ match event {
                            //    EpochFlag::AntennaBeingMoved => {
                            //        td {
                            //            : "Antenna being moved"
                            //        }
                            //    },
                            //    _ => {},
                            //}
                            //        }
                            //td {
                            //    @ match event {
                            //        EpochFlag::NewSiteOccupation => {
                            //            : "New Site Occupation"
                            //        }
                            //        EpochFlag::ExternalEvent => {
                            //            : "External Event"
                            //        }
                            //        _ => {
                            //            : "Other"
                            //        }
                            //    }
                            //}
                        }
                    }
                }
            }
        }
    }
}

/*
 * Epoch Epoch completion,
 * defined as at least 1 Sv with PR + PH observed on both L1 and
 * "rhs" signal,
 * also SNR condition for both signals above current mask
 */
fn report_epoch_completion(
    total: usize,
    total_with_obs: usize,
    complete: &Vec<(Carrier, usize)>,
) -> Box<dyn RenderBox + '_> {
    box_html! {
        table(class="table is-bordered") {
            thead {
                th {
                    : "Epochs"
                }
            }
            tbody {
                tr {
                    th {
                        : "Total#"
                    }
                    td {
                        : total.to_string()
                    }
                }
                tr {
                    th {
                        : "w/ observations"
                    }
                    td {
                        : format!("{} ({}%)", total_with_obs, total_with_obs * 100 / total)
                    }
                }
                tr {
                    th {
                        : "Complete"
                    }
                    @ for (signal, count) in complete {
                        td {
                            b {
                                : format!("L1/{}", signal)
                            }
                            p {
                                : count.to_string()
                            }
                            b {
                                : format!("{}%", count * 100/total)
                            }
                            //: format!("<b>L1/{}</b>: {} (<b>{}%</b>)", signal, count, count *100/total)
                        }
                    }
                }
            }
        }
    }
}

/*
 * SNR analysis report
 */
fn report_snr_analysis(
    min: (Sv, Epoch, Snr),
    max: (Sv, Epoch, Snr),
    ssi_stats: &HashMap<Observable, (f64, f64, f64)>,
) -> Box<dyn RenderBox + '_> {
    box_html! {
        tr {
            th {
                : "Minimum"
            }
            td {
                p {
                    : min.0.to_string()
                }
                p {
                    : format!("{:e}", min.2)
                }
                p {
                    : format!("@{}", min.1)
                }
            }
        }
        tr {
            th {
                : "Best"
            }
            td {
                p {
                    : max.0.to_string()
                }
                p {
                    : format!("{:e}", max.2)
                }
                p {
                    : format!("@{}", max.1)
                }
            }
        }
        tr {
            th {
                : "SSI Statistics"
            }
            td {
                : report_ssi_statistics(ssi_stats)
            }
        }
    }
}

/*
 * Reports statistical analysis results for SSx observations
 */
fn report_ssi_statistics(
    ssi_stats: &HashMap<Observable, (f64, f64, f64)>,
) -> Box<dyn RenderBox + '_> {
    box_html! {
        table(class="table is-bordered") {
            thead {
                tr {
                    td {
                        : ""
                    }
                    @ for (signal, _) in ssi_stats {
                        th {
                            : signal.to_string()
                        }
                    }
                }
            }
            tbody {
                tr {
                    th {
                        : "Mean"
                    }
                    @ for (_, (mean, _, _)) in ssi_stats {
                        td {
                            : format!("{:.3} dB", mean)
                        }
                    }
                }
                tr {
                    th {
                        : "Deviation" // (&#x03C3;)"
                    }
                    @ for (_, (_, sigma, _)) in ssi_stats {
                        td {
                            : format!("{:.3} dB", sigma)
                        }
                    }
                }
                tr {
                    th {
                        : "Skewness" //(&#x03BC; / &#x03C3;&#x03B3;)"
                    }
                    @ for (_, (_, _, sk)) in ssi_stats {
                        td {
                            : format!("{:.3}", sk)
                        }
                    }
                }
            }
        }
    }
}

fn derivative_dt(data: Vec<(Epoch, f64)>) -> Vec<(Epoch, f64)> {
    let mut acc = 0.0_f64;
    let mut prev_epoch: Option<Epoch> = None;
    let mut ret: Vec<(Epoch, f64)> = Vec::new();
    for (epoch, value) in data {}
    ret
}

#[derive(Debug, Clone)]
/// OBS RINEX specific QC analysis.  
/// Full OBS RINEX analysis requires both the "obs" and "processing" features.
pub struct QcObsAnalysis {
    /// Identified Observables
    observables: Vec<String>,
    /// Identified Signals
    signals: Vec<Carrier>,
    /// Codes that were idenfitied
    codes: Vec<String>,
    /// true if doppler observations were identified
    has_doppler: bool,
    /// CS anomalies
    cs_anomalies: Vec<Epoch>,
    /// Epochs where power failures took place, and their duration
    power_failures: Vec<Epoch>,
    /// Other abnormal events, by chronological epochs
    other_anomalies: Vec<(Epoch, EpochFlag)>,
    /// Total number of epochs identified
    total_epochs: usize,
    /// Epochs with at least 1 observation
    total_with_obs: usize,
    /// Complete epochs, with respect to given signal
    complete_epochs: Vec<(Carrier, usize)>,
    #[cfg(feature = "obs")]
    /// Min. SNR (sv @ epoch)
    min_snr: (Sv, Epoch, Snr),
    #[cfg(feature = "obs")]
    /// Max. SNR (sv @ epoch)
    max_snr: (Sv, Epoch, Snr),
    #[cfg(feature = "obs")]
    /// SSi statistical analysis (mean, stddev, skew)
    ssi_stats: HashMap<Observable, (f64, f64, f64)>,
    #[cfg(feature = "obs")]
    /// RX clock drift
    clock_drift: Vec<(Epoch, f64)>,
}

impl QcObsAnalysis {
    pub fn new(rnx: &Rinex, opts: &QcOpts) -> Self {
        let doppler_obs = rnx.observable().filter(|obs| obs.is_doppler_observable());

        let mut observables: Vec<String> = rnx.observable().map(|obs| obs.to_string()).collect();

        let mut signals: Vec<_> = rnx.carrier().unique().collect();
        let mut codes: Vec<_> = rnx.code().map(|c| c.to_string()).collect();

        let cs_anomalies: Vec<_> = rnx
            .epoch_anomalies()
            .filter_map(|(e, flag)| {
                if flag == EpochFlag::CycleSlip {
                    Some(e)
                } else {
                    None
                }
            })
            .collect();

        let power_failures: Vec<_> = rnx
            .epoch_anomalies()
            .filter_map(|(e, flag)| {
                if flag == EpochFlag::PowerFailure {
                    Some(e)
                } else {
                    None
                }
            })
            .collect();

        let other_anomalies: Vec<_> = rnx
            .epoch_anomalies()
            .filter_map(|(e, flag)| {
                if flag != EpochFlag::PowerFailure && flag != EpochFlag::CycleSlip {
                    Some((e, flag))
                } else {
                    None
                }
            })
            .collect();

        let mut total_epochs = rnx.epoch().count();
        let mut epoch_with_obs: Vec<Epoch> = Vec::new();

        let mut complete_epochs: HashMap<Carrier, usize> = HashMap::new();

        let mut min_snr = (Sv::default(), Epoch::default(), Snr::DbHz54);
        let mut max_snr = (Sv::default(), Epoch::default(), Snr::DbHz0);

        let mut ssi_stats: HashMap<Observable, (f64, f64, f64)> = HashMap::new();

        if let Some(rec) = rnx.record.as_obs() {
            let mask: Filter = Filter::from(MaskFilter {
                operand: MaskOperand::Equals,
                item: TargetItem::ClockItem,
            });
            let clk_data = rec.filter(mask);
        }

        if let Some(r) = rnx.record.as_obs() {
            total_epochs = r.len();
            for ((epoch, flag), (clk, svs)) in r {
                for (sv, observables) in svs {
                    if !observables.is_empty() {
                        if !epoch_with_obs.contains(&epoch) {
                            epoch_with_obs.push(*epoch);
                        }
                    }

                    for (observable, observation) in observables {
                        if let Some(snr) = observation.snr {
                            if snr < min_snr.2 {
                                min_snr.0 = *sv;
                                min_snr.1 = *epoch;
                                min_snr.2 = snr;
                            }
                            if snr > max_snr.2 {
                                max_snr.0 = *sv;
                                max_snr.1 = *epoch;
                                max_snr.2 = snr;
                            }
                        }
                    }
                }
            }
            /*
             * Now that signals have been determined,
             * determine observation completion
             */
            for (_, (_, svs)) in r {
                let mut complete: HashMap<Carrier, bool> = HashMap::new();
                for (sv, observables) in svs {
                    for (observable, observation) in observables {
                        if !observable.is_phase_observable() {
                            if !observable.is_pseudorange_observable() {
                                continue;
                            }
                        }
                        /*
                         * SNR condition
                         */
                        if let Some(snr) = observation.snr {
                            if snr < Snr::from(opts.min_snr_db) {
                                continue; // not to be considered
                            }
                        } else {
                            if observable.is_phase_observable() {
                                continue; // phase should have SNR information attached to it
                            }
                        }

                        /*
                         * Signal condition
                         */
                        let carrier_code = &observable.to_string()[1..2];
                        if carrier_code == "1" {
                            // we only search for other signals
                            continue;
                        }

                        let _code = observable.code().unwrap();
                        let carrier = observable.carrier(sv.constellation).unwrap();
                        if let Some(complete) = complete.get_mut(&carrier) {
                            if !*complete {
                                for k_code in carrier::KNOWN_CODES.iter() {
                                    if !k_code.starts_with("1") {
                                        continue; // we're looking for a "1" reference
                                    }
                                    let to_find = match observable.is_phase_observable() {
                                        true => "C".to_owned() + k_code,  // looking for PR
                                        false => "L".to_owned() + k_code, // looking for PH
                                    };
                                    for (observable, _observation) in observables {
                                        if observable.to_string() == to_find {
                                            *complete = true;
                                        }
                                    }
                                    if *complete {
                                        break;
                                    }
                                }
                            }
                        } else {
                            complete.insert(carrier, false);
                        }
                    }
                }
                for (carrier, completed) in complete {
                    if completed {
                        if let Some(count) = complete_epochs.get_mut(&carrier) {
                            *count += 1;
                        } else {
                            complete_epochs.insert(carrier, 1);
                        }
                    }
                }
            }

            /*
             * SSI statistical analysis
             */
            let mut mean_ssi: HashMap<_, _> = r.mean_observable();
            mean_ssi.retain(|obs, _| obs.is_ssi_observable());
            for (obs, mean) in mean_ssi {
                ssi_stats.insert(obs.clone(), (mean, 0.0_f64, 0.0_f64));
            }

            let mut stddev_ssi: HashMap<_, _> = r.mean_observable(); // TODO r.stddev_observable();
            stddev_ssi.retain(|obs, _| obs.is_ssi_observable());
            for (obs, stddev) in stddev_ssi {
                if let Some((_, dev, _)) = ssi_stats.get_mut(&obs) {
                    *dev = stddev;
                }
            }

            let mut skew_ssi: HashMap<_, _> = r.mean_observable(); // TODO r.skewness_observable();
            skew_ssi.retain(|obs, _| obs.is_ssi_observable());
            for (obs, skew) in skew_ssi {
                if let Some((_, _, sk)) = ssi_stats.get_mut(&obs) {
                    *sk = skew;
                }
            }
        }
        /*
         * sort, prior reporting
         */
        codes.sort();
        observables.sort();
        signals.sort();

        Self {
            codes,
            signals,
            observables,
            has_doppler: doppler_obs.count() > 0,
            cs_anomalies,
            power_failures,
            other_anomalies,
            total_epochs,
            total_with_obs: epoch_with_obs.len(),
            complete_epochs: {
                let mut ret: Vec<(Carrier, usize)> =
                    complete_epochs.iter().map(|(k, v)| (*k, *v)).collect();
                ret.sort();
                ret
            },
            min_snr,
            max_snr,
            ssi_stats,
            clock_drift: {
                let rx_clock: Vec<_> = rnx
                    .recvr_clock()
                    .map(|((e, flag), value)| (e, value))
                    .collect();
                let rx_clock_drift: Vec<(Epoch, f64)> = derivative_dt(rx_clock);
                let mov = Averager::mov(opts.clock_drift_window);
                mov.eval(rx_clock_drift)
            },
        }
    }
}

use crate::qc::HtmlReport;
use horrorshow::RenderBox;

impl HtmlReport for QcObsAnalysis {
    fn to_html(&self) -> String {
        unreachable!("never used by itself")
    }
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            tr {
                th {
                    : "Signals"
                }
                td {
                    : report_signals(&self.signals)
                }
            }
            tr {
                th {
                    : "Codes"
                }
                td {
                    : pretty_array(&self.codes)
                }
            }
            tr {
                th {
                    : "Observables"
                }
                td {
                    : pretty_array(&self.observables)
                }
            }
            tr {
                th {
                    : "Has Doppler"
                }
                @ if self.has_doppler {
                    td {
                        : "True"
                    }
                } else {
                    td {
                        : "False"
                    }
                }
            }
            tr {
                table {
                    : report_anomalies(&self.cs_anomalies, &self.power_failures, &self.other_anomalies)
                }
            }
            tr {
                table {
                    : report_epoch_completion(self.total_epochs, self.total_with_obs, &self.complete_epochs)
                }
            }
            tr {
                table {
                    thead {
                        th {
                            : "SNR"
                        }
                    }
                    tbody {
                        : report_snr_analysis(self.min_snr, self.max_snr, &self.ssi_stats)
                    }
                }
            }
        }
    }
}

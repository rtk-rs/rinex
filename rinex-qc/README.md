RINEX / GNSS QC
===============

The `Qc` library, standing for Quality Control, is a GNSS post processing core library.  
It is capable of answering the demanding tasks of precise navigation,
and other similar GNSS post processing tasks.

## Supported File formats

The `Qc` library currently manages many formats, and more may be introduced
in the future.

The following RINEX formats are supported:

- Observation RINEX
- Navigation RINEX
- Meteo RINEX
- IONEx

Other supported formats:

- SP3

The library does not support the following format (as of today):

- DORIS RINEX

## Crate features

This library has many features:

- `flate2` unlocks Gzip decompression native support
- `sp3` unlocks the SP3 format support
- `nav` unlocks post processed Navigation and a few methods
to bridge with the ANISE library
- `cggtts` + `nav` unlocks post processed CGGTTS solutions solver

## Workspace

A session is tied to a Workspace, defined in the Configuration script.  
When deploying and working, `QcContext` needs write access to the entire workspace.

## Deployment

`QcContext` deployment is a complex yet infaillible task.
It will only fail only internal core library major failures that we are not responible for.
If you can access the Internet daily, `QcContext` will deploy with the highest precision `ITRF93` 
frame model. If Internet access is not feasible, it will rely on lower precision offline model.

The `Qc` library uses the RUST Logger internally, it will most notably let you know
how you could "enhance" your input data.

## RINEX input

Stack any supported RINEX to form a complex dataset very easily:

```rust
use rinex_qc::prelude::*;

// default setup
let cfg = QcConfig::default();

// Deployment 
let mut ctx = QcContext::new(cfg)
    .unwrap_or_else(|e| panic!("ctx deployment failure: {}", e));

ctx.load_file("../test_resources/OBS/V3/DUTH0630.22O")
    .unwrap();

assert!(ctx.has_observations());
```

## SP3 input

When built with the `sp3` feature, SP3 data may be loaded into the pool as well.
Standard SP3 data is always indexed correctly in the pool (by publisher Agency):

```rust
use rinex_qc::prelude::*;

// default setup
let cfg = QcConfig::default();

// Deployment 
let mut ctx = QcContext::new(cfg)
    .unwrap_or_else(|e| panic!("ctx deployment failure: {}", e));

ctx.load_gzip_file("../test_resources/SP3/COD0MGXFIN_20230500000_01D_05M_ORB.SP3.gz")
    .unwrap();

assert!(ctx.has_precise_orbits());
```

## Gzip files

Build the library with `flate2` feature to support gzip compressed files natively.
The file extension must be `.gz` for this to work correctly. This applies to any
file format supported by the library:

```rust
use rinex_qc::prelude::*;

// default setup
let cfg = QcConfig::default();

// Deployment 
let mut ctx = QcContext::new(cfg)
    .unwrap_or_else(|e| panic!("ctx deployment failure: {}", e));

ctx.load_gzip_file("../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// load_file dispatches ".gz" files to the gzip loader as well
ctx.load_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    .unwrap();

assert!(ctx.has_observations());
assert!(ctx.has_navigation_data());
```

## Analysis

To analyze your dataset, simply invoke the `QcContext.analyze()` method.   
The analysis to be performed is highly dependent on the provided data
and on the configuration script. It can be either very quick
or very long. Especially when post-processed navigation solutions are requested.

`QcContext` analysis is infaillible. The complexity is the only variation.

## Reporting

Once analysis has been performed, you can generate a report.
We currently support the HTML format to render the analysis report. 
Like analysis synthesis, reporting is always feasible.

Example:

```rust
use rinex_qc::prelude::*;

// default setup
let cfg = QcConfig::default();

// Deploy
let mut ctx = QcContext::new(cfg)
    .unwrap();

ctx.load_gzip_file(
    "../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    .unwrap();

// Analyze and render the report (HTML)
let analysis = ctx.analyze();

let html = analysis.render().into_string();
assert!(!html.is_empty());
```

## Custom chapters

The `Qc` report can be enhanced with custom chapters, that only need you to provide the rendition implementation.
This is work in progress.

## Post processed navigation

The `Qc` library is able to perform the challenging task of precise navigation,
in just a few lines of code. All you need to do is provide a compatible setup.
Refer to the report summary to understand if you setup is compatible.  

In the folllowing example, we provide a BRDC navigation compatible setup.
The solver resolves one solution per epoch of the rover observations:
we only resolve the first epochs here.

```rust
use rinex_qc::prelude::*;

// default setup
let cfg = QcConfig::default();

// deploy
let mut ctx = QcContext::new(cfg)
    .unwrap();

// stack a RINEX
ctx.load_gzip_file(
    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// stack a BRDC RINEX
ctx.load_gzip_file(
    "../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    .unwrap();

// select the rover
let rover = ctx.rover_observations_meta()
    .next()
    .unwrap()
    .clone();

// Deploy a solver
let solver = ctx.nav_pvt_solver(RTKConfig::default(), &rover, None)
    .unwrap();

// Collect the first solutions
for solution in solver.take(3) {
    match solution {
        Ok(pvt) => {
            let (x_m, y_m, z_m) = pvt.pos_m;
        },
        Err(e) => {
            // this epoch could not be resolved
        },
    }
}
```

## KML, GPX tracks

Forming KML or GPX tracks from the navigation solutions is work in progress
and not available yet.

## CGGTTS tracker and solutions solver

The `Qc` library is designed to perform the challenging task of precise timing resolution
in just a few lines of code as well. Instead of deploying the `NavPvtSolver`, prefer
the `NavCggttsSolver` which is dedicated to CGGTTS solutions solving.

Any navigation compatible setup is CGGTTS compatible by definition.
The CGGTTS solver is currently work in progress: it resolves the navigation
solutions but does not form tracks yet, every item it yields is an error.

```rust
use rinex_qc::prelude::*;
use rinex::prelude::Duration;

// default setup
let cfg = QcConfig::default();

// deploy
let mut ctx = QcContext::new(cfg)
    .unwrap();

// stack a RINEX
ctx.load_gzip_file(
    "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    .unwrap();

// stack a BRDC RINEX
ctx.load_gzip_file(
    "../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    .unwrap();

// select the rover
let rover = ctx.rover_observations_meta()
    .next()
    .unwrap()
    .clone();

// Deploy a solver: the reference position of the RINEX is used
let solver = ctx.nav_cggtts_solver(
    RTKConfig::default(),
    &rover,
    None,
    Duration::from_seconds(780.0),
).unwrap();

// Collect the first solutions
for track in solver.take(3) {
    assert!(track.is_err(), "track fitting is not implemented yet");
}
```

## Precise Point Positioning

The configuration lets you express your preferences regarding the orbital
and clock sources to use in PPP scenarios. The navigation solver currently
resolves from broadcast ephemeris only: SP3 files are loaded and indexed
but not used by the solver yet, and Clock RINEX is not a supported input yet.

```rust
use rinex_qc::prelude::*;
use rinex_qc::cfg::{QcPreferedClock, QcPreferedOrbit, QcPreferedSettings};

// prefer SP3 orbits and clocks
let cfg = QcConfig::default()
    .with_preferences(QcPreferedSettings {
        orbit_source: QcPreferedOrbit::SP3,
        clk_source: QcPreferedClock::SP3,
        ..Default::default()
    });

// deploy
let mut ctx = QcContext::new(cfg)
    .unwrap();

// stack SP3
ctx.load_gzip_file(
    "../test_resources/SP3/GRG0MGXFIN_20201770000_01D_15M_ORB.SP3.gz")
    .unwrap();

assert!(ctx.has_precise_orbits());
```

## PPP Guru

Now for all PPP Gurus out there, we're still not quite there yet.  
The stacking and exploitation of `ANTex` is work in progress.

Integrating Navigation solutions
================================

You have two options to integrate Nav solutions to your Qc report:

1. Create your own custom chapter that works from the solutions you just resolved,
and attach it to your report. This is how we used to do it, and it is still viable
2. Request the report synthesizer, through the Config script, to attach
the solutions directly for you. In this case the Config script is all we have to
render and navigate, so it must integrate the RTK config in case it needs to be customized!
refer to the chapters about the Config script.

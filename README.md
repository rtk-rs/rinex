RINEX 
=====

[![Rust](https://github.com/georust/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/rust.yml)
[![Rust](https://github.com/georust/rinex/actions/workflows/daily.yml/badge.svg)](https://github.com/georust/rinex/actions/workflows/daily.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/)
[![crates.io](https://img.shields.io/crates/d/rinex.svg)](https://crates.io/crates/rinex)

[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/georust/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/georust/rinex/blob/main/LICENSE-MIT) 

Rust framework to parse [RINEX](https://en.wikipedia.org/wiki/RINEX), 
SP3 files, and process GNSS data for navigation and much more.

Use the [Github interface](https://github.com/georust/rinex/issues) to report issues.    
Reach out to us on [Discord](https://discord.gg/Fp2aape).  
To keep track of the latest developments, read the [Changelog](https://github.com/georust/rinex/main/CHANGELOG.md)
and keep an eye on the `#rinex` channel on Discord.

## Advantages :rocket: :artificial_satellite:

- Render High level reports from any supported file format
  - including Geodetic survey results
- Integrated PPP solver
- Special CGGTTS solutions solver
- Open sources: read and access all the code!
- Complete set of examples and tutorials within the repo
- All modern GNSS constellations, codes and signals
  - Surveying with GPS, Galileo, BeiDou and QZSS
- Time scales: GPST, QZSST, BDT, GST, UTC, TAI
- Efficient seamless compression and decompression
- RINEX V4 support, up to revision 4.02
- All RINEX formats supported (see following table), including
 - High Precision Clock products (for PPP)
 - IONEX (Ionosphere TEC maps)
 - DORIS (special ground station evaluation from space)
- [SP3 (High Precision Products)](https://docs.rs/sp3/1.0.7/sp3/) also supported (for PPP)
- Many pre-processing algorithms
- Several file operations: merging, splitting, file batch synthesis..

## Disadvantages :warning:

- Navigation is currently not feasible with Glonass and IRNSS (applications/ ppp solver).
- QZSS navigation has not been tested yet
- Enhanced navigation with SBAS is not 100% feasible yet
- RTK (differential navigation) is not 100% feasible yet
- Our applications do not accept proprietary formats like Septentrio for example.
Some solutions may be developped in the future.
- BINEX support is currently partial and work in progress.

## Citation and referencing

If you need to reference this work, please use the following model:

`GeoRust RINEX Team (2024), RINEX: analysis and processing (Apache-2/MIT), https://georust.org`

Getting started
===============

[Follow our tutorials](./tutorials) to get started.   

We recommend starting with basic examples and following the topics you are interested in.

Relevant Ecosystems
===================

* [IGS Network](https://network.igs.org/): browse and monitor any IGS station status
* [Nyx-space](https://github.com/nyx-space/nyx): Navigation and Orbital calculations in Rust
* [Hifitime](https://github.com/nyx-space/hifitime): Precise Time and Timescale support in Rust
* [CGGTTS](https://github.com/gwbres/cggtts): Common View Time Transfer file format, in Rust
* [Geo](https://github.com/georust/geo): Geospatial primitives and algorithms, in Rust
- [RTK-RS](https://github.com/rtk-rs/gnss-rtk): Precise Positioning (calculations) in Rust
* [GNSS](https://github.com/rtk-rs/gnss) in Rust

Repo architecture
=================

This repo holds everything for GNSS post processing.
Including official Rust libraries, some applications,
tutorials on how to use the applications and data sets,
mostly for testing and demonstration purposes.

### Applications

* [`BIN2RNX`](bin2rnx/) is an application to collect a BINEX stream into RINEX files.
* [`CRX2RNX`](crx2rnx/) is a CRINEX decompresor (Compact RINEX to RINEX).
It is a small application that may be operated by itself, or be combined to `rinex-cli` for a complete 
processing pipeline.
* [`RINEX-Cli`](rinex-cli/) is our main application.  
It is a Cli and is not a GUI, which will be developped later (not a current pending topic).   
If you want to see a GUI version happen sooner, feel free to contact us and join the effort.   
This application combines some of `teqc` and `anubis` features.   
It allows post processed navigation like `rtklib` and integrates a special CGGTTS solutions solver.  
The analysis report is currently rendered in HTML.  
The application is auto-generated for a few architectures, you can directly
[download it from Github.com](https://github.com/georust/rinex/releases)
* [`RNX2BIN`](rnx2bin/) dumps one RINEX or CRINEX into a binary file
made of BINEX Messages.
* [`RNX2CRX`](rnx2crx/) is a RINEX compressor (RINEX to Compact RINEX).
It is a light application that you can combined to `rinex-cli` for a complete workflow.
* [`UBX2RNX`](ublox-rnx/) is an application to generate RINEX files from Ublox receivers.   
This application is currently work in progress.

### Libraries

* [`BINEX`](binex/) BINEX message Encoding and Decoding library
* [`Qc-Traits`](qc-traits/) is a low level library that is shared
between our core libraries to permit consistant behaviors.
* [`RINEX`](rinex/) provides RINEX parsing, formatting and CRINEX support.
It allows post processing of all these file formats
* [`RINEX-Qc`](rinex-qc/) is a our GNSS post processing library.
It allows considering a complex fileset of RINEX, possibly enhanced with
SP3. It generates a geodetic report from all of that.
* [`SINEX`](sinex/) is core library dedicated to this special format.
* [`SP3`](sp3/) High Precision Orbits (by IGS) parsing. 
It allows post processing for PPP.

### Other 

* [`logs`](logs/) is dedicated to store sessions log, if you work within this workspace directly.
* [`tutorials`](tutorials/) will help you get started rapidly. The environment is
compatible with the latest release.
* [`tools`](tools/) are utility scripts and development tools

RINEX-Cli
=========

`rinex-cli` is our main application. Like all applications contained in this repo, it is automatically
generated [upon Release](https://github.com/georust/rinex/releases).

`rinex-cli` supports many options that are closely tied to the [Qc options](./rinex-qc):

- `nav`: will enable post processed navigations
- `cggtts`: enables the special CGGTTS solutions
- `kml`: allows to render the PPP solutions as KML tracks
- `gpx`: allows to render the PPP solutions as GPX tracks

Make sure to read how to [activate the application logs](./tutorials/Logs.md) when
performing advanced operations.

Formats & revisions
===================

The `RINEX` lib supports RINEX V4 up to revision 4.02, that includes the new navigation frames.  
It also supports IONEX and Clock RINEX in their latest revisions. 

The `SP3` lib supports rev D.

File format and applications
============================

This table summarizes all supported formats, how they are managed, what they permit,
and whether the applications supports them or not.

`Library Indexing`: gives how this dataset is indexed in the respective core library.   
`Qc Indexing`: gives how this dataset is indexed and managed by the [Qc library](rinex-qc/).  
`TimeScale`: gives what [TimeScale](https://docs.rs/hifitime/latest/enum.TimeScale.html) may be encountered 
during time domain indexing of this file format.   

:heavy_check_mark: all revisions are supported.   
:construction: : work in progress.  
__CLI__ : supported by the [Qc Library](./rinex-qc)   
__CLI__ + :chart_with_upwards_trend: [Qc Reporting](./rinex-qc) will generate one or several graphs  

| Type                               | Parser              | Writer              |  CLI                                           | Content                                         | Library Indexing     | Qc Indexing    | Timescale  |
|----------------------------|---------------------|---------------------|------------------------------------------------|------------------------------------------------------------------------------|----------------------| ---------------| ------------------------------------------------------------------------------|  
| Observation RINEX (OBS)            | :heavy_check_mark:  | :heavy_check_mark:  | :heavy_check_mark: :chart_with_upwards_trend:  | Phase, Pseudo Range, Doppler, SSI                               | [ObsKey]()           | [ObsSorting]() | [Any GNSS](https://docs.rs/hifitime/latest/hifitime/enum.TimeScale.html)           |
| Compact Observation RINEX (CRINEX) | :heavy_check_mark:  | :heavy_check_mark:  | :heavy_check_mark: :chart_with_upwards_trend:  | Phase, Pseudo Range, Doppler, SSI                               | [ObsKey]()           | [ObsSorting]() | [Any GNSS](https://docs.rs/hifitime/latest/hifitime/enum.TimeScale.html)           |
| Navigation RINEX (NAV/BRDC)        | :heavy_check_mark:  | :construction:     | :heavy_check_mark:  :chart_with_upwards_trend:  | Ephemerides, Ionosphere Models, System Time, Earth Orientations | [NavKey]()           | [NavSorting]() | [SV](https://docs.rs/gnss-rs/latest/gnss_rs/sv/struct.SV.html) system time         |
| Ionosphere Maps (IONEX)            | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: :chart_with_upwards_trend:    | Ionosphere TEC projection                                       | [IonexKey]()         | :construction: | [UTC](https://docs.rs/hifitime/latest/hifitime/enum.TimeScale.html#variant.UTC)    |
| Meteo RINEX                        | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: :chart_with_upwards_trend:    | Meteo sensor measurements                                       | [MeteoKey]()         | :construction: | [UTC](https://docs.rs/hifitime/latest/hifitime/enum.TimeScale.html#variant.UTC)    |
| SP3                                | :heavy_check_mark: | :construction:      | :heavy_check_mark: :chart_with_upwards_trend:   | High precision orbital state                                    | [SP3Key]()           | [SP3Sorting]() | [Any GNSS](https://docs.rs/hifitime/latest/hifitime/enum.TimeScale.html)           |
| DORIS (special RINEX)              | :heavy_check_mark: | :heavy_check_mark: | :construction:                                   | High precision DORIS stations observation from space           | [DorisKey]()          | :construction: | [TAI](https://docs.rs/hifitime/latest/hifitime/enum.TimeScale.html#variant.TAI)    |
| BINEX (Binary RINEX)               | partial (a)       | :construction:    | :construction:                                     | Binary messages                                | [BINEX frames](https://docs.rs/binex/latest/binex/prelude/enum.Record.html) | :construction: | :construction: | [Any GNSS](https://docs.rs/hifitime/latest/hifitime/enum.TimeScale.html)           |

BINEX (a): some frames are supported, not all of them. Refer to [BINEX](./binex).

Other formats
=============

| Type | Parser             | Writer                          | CLI                                           | Content                         | Record Iteration | Timescale  |
| ---- | ------------------ | ------------------------------- | --------------------------------------------- | ------------------------------- | ---------------- | ---------- |

File name Restrictions
======================

| Format                 | Restriction              |
| ---------------------- | ------------------------ |
| RINEX                  | :heavy_minus_sign:       |
| CRINEX                 | :heavy_minus_sign:       |
| gzip compressed RINEX  | Name must end with `.gz` |
| gzip compressed CRINEX | Name must end with `.gz` |
| .Z compressed RINEX    | Not supported            |
| DORIS RINEX            | :heavy_minus_sign:       |
| gzip compressed DORIS  | Name must end with `.gz` |
| .Z compressed DORIS    | Not supported            |
| SP3                    | :heavy_minus_sign:       |
| gzip compressed SP3    | Name must end with `.gz` |
| .Z compressed SP3      | Not supported            |
| BINEX                  | :heavy_minus_sign:       |
| UBX                    | :heavy_minus_sign:       |

:heavy_minus_sign: No restrictions: file names do not have to follow naming conventions.  

Non readable formats :construction:
===================================

`RINEX-Cli` will soon accept more than readable data.

| Format |     Status     |                     Application                      |
| :----: | :------------: | :--------------------------------------------------: |
|  UBX   | :construction: | Convert your UBX data to RINEX to later post process |
|        | :construction: | Convert your GNSS context to UBX (efficient storage) |
| BINEX  | :construction: |    Convert BINEX streams to readable RINEX files     |
|        | :construction: |  Encode RINEX datasets to BINEX (efficient storage)  |
|  RTCM  | :construction: |     Serve your RINEX/SP3 datasets over RTCM I/O      |

Special Thanks
==============

These tools would not exist without the great libraries written by C. Rabotin, 
[check out his work](https://github.com/nyx-space).  

Some features would not exist without the invaluable help of J. Lesouple, through
our countless discussions. Check out his 
[PhD manuscript (french)](http://perso.recherche.enac.fr/~julien.lesouple/fr/publication/thesis/THESIS.pdf?fbclid=IwAR3WlHm0eP7ygRzywbL07Ig-JawvsdCEdvz1umJJaRRXVO265J9cp931YyI)

Contributions
=============

Contributions are welcomed, do not hesitate to open new issues
and submit Pull Requests through Github.

If you want to take part in active developments, check out our [contribution guidelines and hints](CONTRIBUTING.md) to navigate this library quicker.

CONTRIBUTING
============

Hello and welcome on board :wave:

Use the github portal to submit PR, all contributions are welcomed.  
Don't forget to run a quick `cargo fmt` prior any submissions, so the CI/CD does not fail
on coding style "issues".

This crate and ecosystem is part of the [RTK-rs framework and community](https://github.com/rtk-rs).  
Contact us on [Discord](https://discord.gg/duETmeGc).

Crate architecture
==================

For each supported RINEX types, we have one folder named after that format.  
`src/navigation` is one of those. 

The module contains the record type definition. RINEX file contents vary a lot
depending on which type of RINEX we're talking about.  
For complex RINEX formats like Navigation Data, that module will contain all possible inner types.

Other important structures :
- SV and Constellation support is provided by the GNSS lib (gnss-rs)
- `src/epoch/mod.rs`: the Epoch module basically provides
hifitime::Epoch parsing methods, because RINEX describes date in non standard formats.
Also, the `Flag` structure is used to mark Observations (valid or invalid).
- `src/observable.rs`: defines possible observations like raw phase
- `src/carrier.rs`: defines carrier signals in terms of frequency and bandwidth.
It also contains utilities to identify which GNSS signals we're dealing with,
from an `Observable`.
- `src/hatanaka/mod.rs`: the Hatanaka module contains the RINEX Compressor and Decompressor 
- `src/antex/antenna.rs`: defines the index structure of ANTEX format

Navigation Data
===============

Orbit broadcasted parameters are presented in different form depending on the RINEX revisions
and also may differ in their nature depending on which constellation we're talking about.

To solve that problem, we use a dictionary, in the form of `src/db/NAV/orbits.json`,
which describes all fields per RINEX revision and GNSS constellation.

This file is processed at build time, by the build script and ends up as a static 
pool we rely on when parsing a file. 

The dictionary is powerful enough to describe all revision, the default Navigation Message
is `LNAV`: Legacy NAV message, which can be omitted. Therefore, it is only declared 
for modern Navigation messages.

Introducing a new RINEX type
============================

`src/meteo/mod.rs` is the easiest format and can serve as a guideline to follow.

When introducing a new Navigation Data, the dictionary will most likely have to be updated (see previous paragraph).

GNSS Constellations
===================

Supported constellations are defined in the Constellation Module.  
This structure defines both Orbiting and Stationary vehicles.

Adding new SBAS vehicles
========================

To add a newly launched SBAS vehicles, simply add it to the
gnss-rs/data/sbas.json database.

This database is auto integrated to this library to provide
detailed SBAS supports. The only mandatory fields (in the databse) are:
- the "constellation" field
- the SBAS "prn" field (which is 100 + prn number)
- "id": the name of that vehicle, for example "ASTRA-5B"
- "launched\_year": the year this vehicle was launched

Other optional fields are:
- "launched\_month": month ths vehicle was launched
- "launched\_day": day of month this vehicle was launched

We don't support undeployed vehicles (in advance).

Modify or add Navigation Frames
===============================

Navigation frames are desc

The build script is rinex/build.rs.

It is responsible for building several important but hidden structures.

1. Navigation RINEX specs, described by rinex/db/NAV
2. Geostationary vehicles identification in rinex/db/sbas/sbas.json,
that follows the L1-CA-PRN Code assignment specifications (see online specs).
3. rinex/db/SBAS/*.wkt contains geographic definitions for most
standard SBAS systems. We parse them as Geo::LineStrings to
define a contour area for a given SBAS system. This gives one method
to select a SBAS from given location on Earth

External key dependencies
=========================

- `Hifitime` for timing and timescales
- `ANISE` for navigation

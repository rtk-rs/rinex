mod cli;
use cli::Cli;
use rinex::{prelude::*, Error};
fn main() -> Result<(), Error> {
    let cli = Cli::new();
    let input_path = cli.input_path();

    let mut rinex = Rinex::from_file(input_path)?; // parse

    println!("Compressing \"{}\"..", input_path);
    rinex.rnx2crnx_mut();

    // compression attributes
    if cli.crx1() {
        if let Some(obs) = &mut rinex.header.obs {
            if let Some(crx) = &mut obs.crinex {
                crx.version.major = 1; // force to V1
            }
        }
    }
    if cli.crx3() {
        if let Some(obs) = &mut rinex.header.obs {
            if let Some(crx) = &mut obs.crinex {
                crx.version.major = 3; // force to V3
            }
        }
    }
    if let Some(date) = cli.date() {
        let (y, m, d, _, _, _, _) = date.to_gregorian_utc();
        if let Some((hh, mm, ss)) = cli.time() {
            if let Some(obs) = &mut rinex.header.obs {
                if let Some(crx) = &mut obs.crinex {
                    crx.date = Epoch::from_gregorian_utc(y, m, d, hh, mm, ss, 0);
                }
            }
        } else if let Some(obs) = &mut rinex.header.obs {
            if let Some(crx) = &mut obs.crinex {
                crx.date = Epoch::from_gregorian_utc_at_midnight(y, m, d);
            }
        }
    } else if let Some((hh, mm, ss)) = cli.time() {
        let today = Epoch::now().expect("failed to retrieve system time");
        let (y, m, d, _, _, _, _) = today.to_gregorian_utc();
        if let Some(obs) = &mut rinex.header.obs {
            if let Some(crx) = &mut obs.crinex {
                crx.date = Epoch::from_gregorian_utc(y, m, d, hh, mm, ss, 0);
            }
        }
    }

    // output path
    let output_path = match cli.output_path() {
        Some(path) => path.clone(), // use customized name
        _ => {
            if cli.matches.get_flag("short") {
                // short name requested
                rinex.standardized_short_filename(false, None, None).expect(
                    "Input file does not follow naming conventions.
You should use --output then.",
                )
            } else {
                // always prefer standardized & modern format
                rinex.standardized_filename(None).expect(
                    "Input file does not follow naming conventions.
You should use --output then.",
                )
            }
        },
    };

    rinex.to_file(&output_path)?;
    println!("{} generated", output_path);
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::tests::toolkit::doris_check_observables;
    use crate::tests::toolkit::doris_check_stations;

    use crate::prelude::*;
    use std::path::Path;

    #[test]
    #[cfg(feature = "flate2")]
    fn v3_cs2rx18164() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_resources")
            .join("DOR")
            .join("V3")
            .join("cs2rx18164.gz");
        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_gzip_file(fullpath.as_ref()).unwrap(); // verified elsewhere

        doris_check_observables(
            &rinex,
            &["L1", "L2", "C1", "C2", "W1", "W2", "F", "P", "T", "H"],
        );
        doris_check_stations(
            &rinex,
            &[
                "D01  THUB THULE                         43001S005  3   0",
                "D02  SVBC NY-ALESUND II                 10338S004  4   0",
                "D03  YEMB YELLOWKNIFE                   40127S009  4   0",
                "D04  JIWC JIUFENG                       21602S007  3   0",
                "D05  MLAC MANGILAO                      82301S001  3   0",
                "D06  MANB MANILLE                       22006S002  3   0",
                "D07  YASB YARAGADEE                     50107S011  4   0",
                "D08  MSPB MOUNT STROMLO                 50119S004  4   0",
                "D09  ADHC TERRE ADELIE                  91501S005  3   0",
                "D10  BEMB BELGRANO                      66018S002  3   0",
                "D11  RISC RIO GRANDE                    41507S008  3   0",
                "D12  SJVC SAN JUAN                      41508S005  4   0",
                "D13  KRWB KOUROU                        97301S006  4   0",
                "D14  LAOB LE LAMENTIN                   97205S001  3   0",
                "D15  MIAB MIAMI                         49914S003  4   0",
                "D16  MNAC MANAGUA                       41201S002  3   0",
                "D17  GRFB GREENBELT                     40451S178  3   0",
                "D18  STKC ST JOHN'S                     40101S004  4   0",
                "D19  CIDB CIBINONG                      23101S003  4   0",
                "D20  AMVB AMSTERDAM                     91401S005  4   0",
                "D21  KEYC KERGUELEN                     91201S008  4   0",
                "D22  SCSC SANTA CRUZ                    42005S003  4   0",
                "D23  GONC GOLDSTONE                     40405S043  4   0",
                "D24  KIVC KITAB                         12334S007  3   0",
                "D25  MALC MALE                          22901S003  4   0",
                "D26  MAVC MARION ISLAND                 30313S006  4   0",
                "D27  HROC HANGA ROA                     41781S001  4   0",
                "D28  RIMC RIKITEA                       92301S005  4   0",
                "D29  SOFC SOCORRO                       40503S006  3   0",
                "D30  COBB COLD BAY                      49804S004  3   0",
                "D31  MEUB METSAHOVI                     10503S016  4   0",
                "D32  DJIB DJIBOUTI                      39901S003  4   0",
                "D33  MAIB MAHE                          39801S006  4   0",
                "D34  REVC LA REUNION                    97401S004  4   0",
                "D35  PAUB PAPEETE                       92201S010  3   0",
                "D36  KOLB KAUAI                         40424S009  4   0",
                "D37  DIOB DIONYSOS                      12602S012  4   0",
                "D38  GAVC GAVDOS                        12618S002  4  13",
                "D39  GR4B GRASSE                        10002S019  4 -15",
                "D40  HBMB HARTEBEESTHOEK                30302S008  4   0",
                "D41  OWFC OWENGA                        50253S002  4   0",
                "D42  TLSB TOULOUSE                      10003S005  4   0",
                "D43  TSTC TEST-FERMAT                   10003S006  4 -22",
                "D44  LICB LIBREVILLE                    32809S004  3   0",
                "D45  HEMB ST HELENA                     30606S004  4   0",
                "D46  ASEB ASCENSION                     30602S005  3   0",
                "D47  TRJB TRISTAN DA CUNHA              30604S003  4   0",
                "D48  NOXC NOUMEA                        92701S005  4   0",
                "D49  BETB BETIO                         50305S001  3   0",
                "D50  PDOC PONTA DELGADA                 31906S004  4   0",
            ],
        );
    }
}

#[cfg(test)]
mod tests {

    use std::fs::File;
    use std::path::{Path, PathBuf};

    use std::io::{BufReader, BufWriter};

    pub struct TestConstants;

    impl TestConstants {
        pub const PROJECT_ROOT_DIR: &'static str = "../";
        pub const TEST_SAMPLES_SUBDIR: &'static str = "tests/test_files";
        pub const OUTPUT_DIR: &'static str = "output";

        pub fn get_output_path() -> PathBuf {
            Path::new(Self::PROJECT_ROOT_DIR).join(Self::OUTPUT_DIR)
        }

        pub fn get_output_dir_path(output_filename: String) -> PathBuf {
            Path::new(Self::PROJECT_ROOT_DIR)
                .join(Self::OUTPUT_DIR)
                .join(Path::new(&output_filename))
                .to_string_lossy()
                .into_owned()
                .into()
        }

        pub fn take_sample_file(sample_filename: String) -> PathBuf {
            Path::new(Self::PROJECT_ROOT_DIR)
                .join(Self::TEST_SAMPLES_SUBDIR)
                .join(Path::new(&sample_filename))
                .to_string_lossy()
                .into_owned()
                .into()
        }
    }

    use crate::parser::{
        InputParserFormat, OutputParserFormat, parse_input_and_serialize_via_trait,
    };

    #[test]
    fn test_convert_csv_to_csv() {
        let input_file = File::open(Path::new(&TestConstants::take_sample_file(
            "example_of_report_bill_1.csv".to_string(),
        )))
        .unwrap();

        let reader_from_file = BufReader::new(input_file);

        let output_dir: &PathBuf = &TestConstants::get_output_path();
        std::fs::create_dir_all(&output_dir).unwrap();
        let outputfile = File::create(Path::new(&TestConstants::get_output_dir_path(
            "test_convert_csv_to_csv.csv".to_string(),
        )))
        .unwrap();

        let output_writer_file = BufWriter::new(outputfile);
        let _result_1 = parse_input_and_serialize_via_trait(
            reader_from_file,
            output_writer_file,
            InputParserFormat::CsvExtraFin,
            OutputParserFormat::CsvExtraFin,
        );
        assert!(_result_1.is_ok());
    }

    #[test]
    fn test_parse_mt940() {
        let output_dir: &PathBuf = &TestConstants::get_output_path();
        std::fs::create_dir_all(&output_dir).unwrap();

        let _result_2 = parse_input_and_serialize_via_trait(
            File::open(Path::new(&TestConstants::take_sample_file(
                "MT_940_oracle.mt940".to_string(),
            )))
            .unwrap(),
            File::create(Path::new(&TestConstants::get_output_dir_path(
                "MT_940_oracle.mt940_to_csv.txt".to_string(),
            )))
            .unwrap(),
            InputParserFormat::Mt940,
            OutputParserFormat::Mt940,
        );

        let _result_4 = parse_input_and_serialize_via_trait(
            File::open(Path::new(&TestConstants::take_sample_file(
                "MT940_github_1.mt940".to_string(),
            )))
            .unwrap(),
            File::create(Path::new(&TestConstants::get_output_dir_path(
                "MT940_github_1.mt940_to_.mt940".to_string(),
            )))
            .unwrap(),
            InputParserFormat::Mt940,
            OutputParserFormat::Mt940,
        );
    }

    #[test]
    fn test_circular_csv_mt940_camt053() {
        let output_dir: &PathBuf = &TestConstants::get_output_path();
        std::fs::create_dir_all(&output_dir).unwrap();

        let _result_12 = parse_input_and_serialize_via_trait(
            File::open(Path::new(&TestConstants::take_sample_file(
                "example_of_report_bill_1.csv".to_string(),
            )))
            .unwrap(),
            File::create(Path::new(&TestConstants::get_output_dir_path(
                "test_circular_csv_mt940_camt053_1.mt940".to_string(),
            )))
            .unwrap(),
            InputParserFormat::CsvExtraFin,
            OutputParserFormat::Mt940,
        );

        let _result_13 = parse_input_and_serialize_via_trait(
            File::open(Path::new(&TestConstants::get_output_dir_path(
                "test_circular_csv_mt940_camt053_1.mt940".to_string(),
            )))
            .unwrap(),
            File::create(Path::new(&TestConstants::get_output_dir_path(
                "test_circular_csv_mt940_camt053_2.xml".to_string(),
            )))
            .unwrap(),
            InputParserFormat::Mt940,
            OutputParserFormat::Camt053,
        );
    }

    #[test]
    fn test_render_extra_csv_to_self() {
        let output_dir: &PathBuf = &TestConstants::get_output_path();
        std::fs::create_dir_all(&output_dir).unwrap();
        let _result_4 = parse_input_and_serialize_via_trait(
            File::open(Path::new(&TestConstants::take_sample_file(
                "MT940_github_1.mt940".to_string(),
            )))
            .unwrap(),
            File::create(Path::new(&TestConstants::get_output_dir_path(
                "MT940_github_1.mt940_to_.csv".to_string(),
            )))
            .unwrap(),
            InputParserFormat::Mt940,
            OutputParserFormat::CsvExtraFin,
        );
    }

    #[test]
    fn test_parse_camt053_to_self() {
        let output_dir: &PathBuf = &TestConstants::get_output_path();
        std::fs::create_dir_all(&output_dir).unwrap();

        let _result_7 = parse_input_and_serialize_via_trait(
            File::open(Path::new(&TestConstants::take_sample_file(
                "camt_053_danske_bank.xml".to_string(),
            )))
            .unwrap(),
            File::create(Path::new(&TestConstants::get_output_dir_path(
                "camt_053_danske_bank_Camt053.xml".to_string(),
            )))
            .unwrap(),
            InputParserFormat::Camt053,
            OutputParserFormat::Camt053,
        );
    }
}

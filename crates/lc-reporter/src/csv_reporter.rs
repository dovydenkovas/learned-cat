use crate::*;
use log::error;
use std::io::Write;

pub struct CsvReporter {
    statistic: Box<dyn Statistic>,
}

impl CsvReporter {
    pub fn new(statistic: Box<dyn Statistic>) -> CsvReporter {
        CsvReporter { statistic }
    }
}

impl Reporter for CsvReporter {
    /// Сохранение результатов тестирования в файл.
    fn save_report(&mut self, filename: PathBuf) {
        // Create output file
        let mut file = match std::fs::File::create(&filename) {
            Ok(f) => f,
            Err(err) => {
                error!("Не могу создать файл {}: {err}", filename.to_str().unwrap());
                std::process::exit(1);
            }
        };

        // Save output file
        for user in &self.statistic.users() {
            for result in &self.statistic.results(user) {
                let out = format!(
                    "{},{},{},{},{}\n",
                    result.testname,
                    result.username,
                    result.start_datetime.to_string(),
                    result.end_datetime.to_string(),
                    result.mark
                );

                print!("{}", out);
                let _ = file.write(out.as_bytes());
            }
            println!();
        }
    }
}

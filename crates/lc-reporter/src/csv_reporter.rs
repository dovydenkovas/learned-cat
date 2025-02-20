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
    fn marks_report(&mut self, filename: PathBuf) {
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

    /// Созранение вариантов пользователя в файл.
    fn variants_report(&mut self, username: &String, testname: &String) {
        let variant_report = self.statistic.variants(username, testname);

        println!(
            "# Результаты теста {} для пользователя {}\n",
            testname, username
        );
        for variant in variant_report {
            println!("## Вариант от {}", variant.start_datetime);
            println!("### Завершен {}", variant.end_datetime);
            println!("### Оценка {}", variant.mark);
            println!("### Вопросы: ");
            for question in variant.questions {
                println!("#### {} ", question.question);
                for answer in question.answers {
                    if answer.is_selected {
                        print!("- [x] ");
                    } else {
                        print!("- [ ] ");
                    }

                    if answer.is_correct {
                        println!(" _{}_", answer.answer);
                    } else {
                        println!(" {}", answer.answer);
                    }
                }
                println!("");
            }
        }
        println!("");
    }
}

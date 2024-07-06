use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::exit;

const DIR_PERMISSIONS: u32 = 0o750;
const FILE_PERMISSIONS: u32 = 0o640;

/// Change file permissions.
///
/// _path_ - path to file or directory.
///
/// _mode_ - unix permissions mask in octal notation.
///
/// # Examples
///
/// ```
/// chmod("/opt/example", 0o770);
/// chmod("settings.toml", 0o664);
/// ```
pub fn chmod(path: &Path, mode: u32) {
    let file = File::open(path).expect("Ошибка доступа к корневой дирректории");
    let metadata = file.metadata().expect("Ошибка чтения прав доступа");
    let mut permissions = metadata.permissions();
    permissions.set_mode(mode);
    file.set_permissions(permissions)
        .expect("Ошибка установки прав доступа");
}

/// Create server files with example configuration
pub fn init_server(path: &Path) {
    println!(" * Создание файлов сервера");
    create_root(path);
    if !path.join("results").exists() {
        std::fs::create_dir(path.join("results")).expect("Ошибка создание дирректории результатов");
        chmod(path.join("results").as_path(), DIR_PERMISSIONS);
    }

    if !path.join("tests").exists() {
        std::fs::create_dir(path.join("tests")).expect("Ошибка создания дирректории тестов");
        chmod(path.join("tests").as_path(), DIR_PERMISSIONS);
    }
    create_settings(path);
    create_example_test(path);
}

/// Create main directory and set permissions.
fn create_root(path: &Path) {
    if !path.exists() {
        if std::fs::create_dir(path).is_err() {
            eprintln!(
                "Ошибка создания корневой дирректории \"{}\".",
                path.to_str().unwrap()
            );
            exit(1);
        }
        chmod(path, DIR_PERMISSIONS);
    }
}

/// Create example settings with all parameters.
fn create_settings(path: &Path) {
    let path = path.join("settings.toml");
    if path.exists() {
        return;
    }
    let example_settings = r#"tests_directory_path = "tests" # Путь к каталогу с файлами тестов
result_path = "results" # Путь к каталогу где должны храниться результаты тестирования
server_address = "127.0.0.1:65001" # Адрес сервера тестирования.
new_file_permissions = 0o644 # Права доступа файла результата (0o - спецификатор восьмиричной системы счисления)

[[test]]
caption="linux" # Название теста (необходимо для запуска теста и поиска файла теста)
questions_number = 2 # Количество вопросов, которые необходимо выбрать для генерации варианта
test_duration_minutes = 5 # Ограничение тестирования по времени
show_results = true # Показывать ли баллы пользователю
allowed_users = ["student1", "student2"] # Имена пользователей, имеющих право выполнять тест
number_of_attempts = 3 # Разрешенное количество попыток
"#;

    let mut file = File::create(&path).expect("Ошибка создания файла настоек");
    file.write(example_settings.as_bytes())
        .expect("Ошибка сохранения файла настроек");

    chmod(path.as_path(), FILE_PERMISSIONS);
}

fn create_example_test(path: &Path) {
    let path = path.join("tests").join("linux.md");
    if path.exists() {
        return;
    }
    let example_settings = r#"Тестирование по командам ОС Linux. Успехов!

# Что делает утилита cat?
* Вызывает кота, который бегает за курсором мыши
+ Выводит содержимое файла
* Это пакетный менеджер, позволяет устанавливать программы
* Явно ничего хорошего
* Такой команды нет


# Как выйти из VIM с сохранением файла?
* Alt+F4
* Закрыть окно
* Ctrl+O
+ ZZ
+ Esc + :wq + Enter
* Никак, надо просто смириться
"#;

    let mut file = File::create(&path).expect("Ошибка создания примера теста");
    file.write(example_settings.as_bytes())
        .expect("Ошибка сохранения примера теста");
    chmod(path.as_path(), FILE_PERMISSIONS);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_perms(path: &Path) -> u32 {
        path.metadata().unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn chmnod_test() {
        let dir = std::env::temp_dir();
        let path = dir.join("chmod_test.txt");
        let _ = File::create(&path).unwrap();
        chmod(path.as_path(), 0o765);

        let perms = get_perms(path.as_path());
        std::fs::remove_file(path).unwrap();

        assert_eq!(perms & 0o777, 0o765);
    }

    #[test]
    fn init_server_test() {
        let dir = std::env::temp_dir();
        let path = dir.join("server_example");
        init_server(path.as_path());

        assert!(path.exists());
        assert!(path.join("settings.toml").exists());
        assert!(path.join("tests").exists());
        assert!(path.join("results").exists());
        assert!(path.join("tests/linux.md").exists());
        assert_eq!(get_perms(path.as_path()), DIR_PERMISSIONS);
        assert_eq!(
            get_perms(path.join("settings.toml").as_path()),
            FILE_PERMISSIONS
        );

        assert_eq!(get_perms(path.join("tests").as_path()), DIR_PERMISSIONS);

        assert_eq!(get_perms(path.join("results").as_path()), DIR_PERMISSIONS);

        assert_eq!(
            get_perms(path.join("tests/linux.md").as_path()),
            FILE_PERMISSIONS
        );
        std::fs::remove_dir_all(path).unwrap();
    }
}

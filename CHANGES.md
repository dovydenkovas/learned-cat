# Изменения

## [v0.2.0]

### Добавлено
- [x] Добавлено логирование запросов, включающее разные уровни.
- [x] В Makefile были добавлены скрипты установки, удаления и обновления программы в среде linux. При использовании systemd добавляется служба learned-cat.
- [x] Написаны тесты для ключевых модулей системы, что повышает её надежность.
- [x] За вопросы с несколькими ответами рассчитывается дробный балл.
- [x] В настройках сервера вместо списка разрешенных пользователей можно указывать путь к текстовому файлу, содержащему перечень пользователей.
При этом имена в файле должны быть перечилены без знаков препинания через пробел или символ переноса строки.
- [x] Варианты тестирования сохраняются в базе данных, их можно экспортировать.

### Изменено
- [x] Результаты тестирования теперь хранятся в sqite базе данных. Это позволяет уменьшить объем потребляемой оперативной памяти и упращает данные, обрабатываемые сервером при выполнении тестирования, что ускоряет скорость работы сервера.

### Исправлено
- [x] Запрет ответа на последний вопрос после завершения теста
- [x] Вывод результата вместо ошибки при завершении времени
- [x] Корректное сообщение при запуске недоступного теста.
- [x] Корректное назначение значений для параметров не указанных в файле настроек


## [v0.1.2]
### Добавлено
- [x] Добавлен минимальный набор тестов.
- [x] Добавлена функция изменения расположения директории сервера с помощью переменной окружения `LEARNED_CAT_PATH`.
- [x] Добавлено выставление прав доступа на каталоги и файлы сервера при инициализации файлов сервера.

### Изменено
- [x] Улучшены аргументы серверного приложения, для лучшего пользовательского опыта.

### Исправлено
- [x] Исправлено зависание системы при повторном запуске теста.
- [x] Исправлена ошибка при выходе на последнем вопросе.

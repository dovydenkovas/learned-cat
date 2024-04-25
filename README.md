# Программа автоматического тестирования в консоли

[![Build](https://github.com/dovydenkovas/sshtest/actions/workflows/rust.yml/badge.svg)](https://github.com/dovydenkovas/sshtest/actions/workflows/rust.yml)

Позволяет проводить тестирование в терминале. Каждый тест состоит из описания, 
вопросов и дополнительных параметров. Каждый вопрос содержит текст вопроса и 
варианты ответов. Поддерживается выбор нескольких вариантов. 

Программа состоит из двух частей - сервера и клиента. Клиент осуществляет 
соединение с сервером, запрашивает необходимый тест и проигрывает его, после 
чего возвращает серверу ответы. Сервер осуществляет считывание файлов тестов,
настроек и результатов тестирования, обслуживание клиентов, проверку тестов и сохранение 
результатов. 

## Описание работы клиета 
Клиент отправляет короткие запросы и получает ответы от сервера. Осуществляет 
две основные функции:
* Получение информации о доступных тестов (при запуске без аргументов или с ключем -l)
* Запуск и проигрывание теста (при запуске с аргументом - именем теста)

При запросе на сервер клиент указывает учетные данные пользователя, чтобы 
получить список тесты, доступные данному пользователю.
При запуске теста сначала выводится описание теста и предложение к началу 
тестирования. В случае согласия пользователя загружается первый вопрос.
Тест считается запущенным с момента получения первого вопроса.
После ответа на предыдущий вопрос ответы отправляются на сервер, а сервер 
присылает следующий вопрос. Ответами на вопросы является одно или несколько
чисел - номеров правильных ответов. Несколько чисел указываются через пробел. 
После ответа на последний вопрос сервер присылает результат или сообщение о 
завершении теста, если публикация результатов отключена в настройках. 
При преждевременном завершении теста можно продолжить тестирование с последнего 
отвеченного вопроса, для этого необходимо заново запустить тест. В случае если
тест ограничен по времени, то после окончания времени пользователь может ответить
на последний вопрос, загруженный до окончания времени, оставшиеся вопросы считаются 
неотвеченными и баллы за них не начисляются, сервер отправляет результат тестирования.


## Описание работы сервера
Сервер реализует следующий функционал:
1. Считывание настроек
2. Парсинг тестов из markdown файлов
3. Открытие/создание файлов ответов
4. Запуск цикла обработки запросов

В рамках взаимодействия с клиентами сервер осуществляет:
1. Проверку доступа пользователя. Пользователь может получить информацию только
о доступных ему тестах и запускать только доступные ему тесты.
2. Выдачу описания теста.
3. Генерацию варианта с учетом перемешивания вопросов и вариантов ответов. 
4. Регистрацию времени начала и времени окончания тестирования
5. Подведение итогов, проверку тестирования.
6. Завершение тестирования при окончании времени.
7. Занесение информации о времени начала тестирования, сгенерированном варианте,
выбранных ответах, полученных баллах и времени завершения каждого теста для каждого пользователя.

Результаты тестирования хранятся в формате toml и содержат самую полную информацию
о заданных вопросах и выбранных пользователем ответах. 
Кроме того запуская сервер с параметром `export-results` можно сохранить краткую
информацию о результатах тестирования в формате csv. 

## Настройка 
Для запуска сервера необходимо создать тесты. Тест представляет собой markdown файл,
состоящий из заголовков, текста и списков. 
В начале файла должен распологаться текст - описание теста, может состоять из 
любого количества строк.

Текст вопроса начинается с заголовка (#), далее следует произвольное количество обычных 
строк. Варианты ответов представлют собой маркерный список. Неправильные ответы 
начинаются с символа `*`, а правильные с `+`.Ответы также могут занимать несколько строк.
Правильных ответов может быть несколько. Пример теста приведен ниже:

```markdown
Тестирование по командам ОС Linux. Успехов!

# Что делает утилита cat?
* Вызывает кота, который бегает за курсором мыши
+ Выводит содержимое файла
* Это пакетный менеджер, позволяет устанавливать программы
* Явно ничего хорошего
* Такой команды нет


# Что делает утилита pacman?
* Запускает игру
* Утилита для удаления файлов
+ Это пакетный менеджер, позволяет устанавливать программы
* Такой команды нет в моем дистрибутиве

```

Настройки сервера храняться в файле `settings.toml`, пример настройки, включающей 
все параметры представлен ниже:

```toml
tests_directory_path = "tests" # Путь к каталогу с файлами тестов
result_path = "results" # Путь к каталогу где должны храниться результаты тестирования 
server_address = "127.0.0.1:65432" # Адрес сервера тестирования. 

[[test]]
caption="linux" # Название теста (необходимо для запуска теста и поиска файла теста)
questions_number = 2 # Количество вопросов, которые необходимо выбрать для генерации варианта
test_duration_minutes = 5 # Ограничение тестирования по времени
show_results = true # Показывать ли баллы пользователю
allowed_users = ["asd"] # Имена пользователей, имеющих право выполнять тест
number_of_attempts = 3 # Разрешенное количество попыток
```

Кроме того на клиенте необходимо задать адрес сервера с помощью переменной 
окружения, напиример:

```sh
export SERVER_ADDRESS=127.0.0.1:65432
```

## Сборка
Для сборки проекта необходим компилятор rustc и менеджер пакетов cargo 
(чаще всего устанавливаются одним пакетом `rust` или аналогичным названием). 
Необходимо клонировать репозиторий и собрать проект:

```bash
make
```
В результате сборки появятся файлы
`sshtest` и `sshtest-daemon`, 
которые представляют клиента и сервер соответственно.


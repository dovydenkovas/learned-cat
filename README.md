[![Build](https://github.com/dovydenkovas/sshtest/actions/workflows/rust.yml/badge.svg)](https://github.com/dovydenkovas/sshtest/actions/workflows/rust.yml)

# Программа автоматического тестирования в консоли
Позволяет проводить тестирование в терминале. Каждый тест состоит из названия,
описания и вопросов. Каждый вопрос содержит текст вопроса и варианты ответов.
Поддерживается выбор нескольких вариантов. 

Программа состоит из двух частей - сервера и клиента. Клиент осуществляет 
соединение с сервером, запрашивает необходимый тест и проигрывает его, после 
чего возвращает серверу ответы. Сервер осуществляет считывание файлов тестов,
настроек и базы данных, обслуживание клиентов, проверку тестов и сохранение 
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
присылает следующий вопрос. Ответами на вопросы является одно или несколько чисел
 - номеров правильных ответов. Несколько чисел указываются через пробел. 
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
3. Открытие/создание базы данных ответов
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


## Настройка 
Для запуска сервера необходимо создать тесты. Тест представляет собой markdown файл,
состоящий из заголовков, текста и списков. Первый заголовок (#) - название теста. 
Должно быть одним словом, без пробелов.
Далее следует описание теста, может состоять из любого количества строк.
Текст вопроса начинается с заголовка (#), далее следует произвольное количество обычных 
строк. Варианты ответов представлют собой маркерный список. Неправильные ответы 
начинаются с символа `*`, а правильные с `+`.Ответы также могут занимать несколько строк.
Правильных ответов может быть несколько. Пример теста приведен ниже:

```markdown
# Математика_1класс
Тест по математике на сложение чисел.

# 1 + 1 = ?
* 0
* 1
+ 2
* 4

# 2 + 3 = ?
* 12
+ 5
* 6
* 7
* 8
* 4

```

Настройки сервера состоят из следующих полей:

-|-
поле | описание
-|-


## Сборка
Для сборки проекта необходим компилятор rustc и менеджер пакетов cargo. 
Необходимо клонировать репозиторий и собрать проект:
```bash
git clone https://github.com/dovydenkovas/sshtest
cd sshtest
cargo build --release
```
В результате сборки появятся файлы
`target/release/sshtest` и `target/release/sshtest-server`, 
которые представлют клиента и сервер соответственно.

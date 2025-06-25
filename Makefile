SETTINGS_PATH="/etc/learned-cat"
DATABASE_PATH="/var/lib/learned-cat"

all:
	@cargo build --release
	@cp target/release/learned-cat learned-cat
	@cp target/release/learned-cat-daemon learned-cat-daemon

install: uninstall add_user all update
	@echo "Установка файлов настроек"
	@sudo cp -r example-config ${SETTINGS_PATH}
	@sudo chown -R learned-cat:learned-cat ${SETTINGS_PATH}
	@sudo chmod 770 ${SETTINGS_PATH}

	@echo "Установка каталога базы данных"
	@sudo mkdir ${DATABASE_PATH}
	@sudo chown -R learned-cat:learned-cat ${DATABASE_PATH}
	@sudo chmod 770 ${DATABASE_PATH}

#@if [[ "systemd" = `ps --no-headers -o comm 1` ]]; then \
#   sudo cp learned-cat.service /etc/systemd/system/learned-cat.service; \
#else \
#	@echo "ПРЕДУПРЕЖДЕНИЕ: Systemd не обнаружена. Используйте свой менеджер инициализации."; \
#	@echo "                Для запуска демона используйте команду /opt/learned-cat/learned-cat-daemon -r"; \
#	@echo "                Администратор демона: learnedcat"; \
#fi;

add_user:
	@echo "Добавляю пользователя learned-cat"
	@sudo useradd -r -s /usr/sbin/nologin -M learned-cat || echo "Пользователь уже существует"

update: add_user all
	@echo "Установка learned-cat"
	@sudo cp learned-cat /usr/bin/learned-cat
	@sudo chown learned-cat:learned-cat /usr/bin/learned-cat
	@sudo chmod 755 /usr/bin/learned-cat

	@echo "Установка learned-cat-daemon"
	@sudo cp learned-cat-daemon /usr/bin/learned-cat-daemon
	@sudo chown learned-cat:learned-cat /usr/bin/learned-cat-daemon
	@sudo chmod 754 /usr/bin/learned-cat-daemon

	@sudo cp completions/* /etc/bash_completion.d/

uninstall:
	@echo "Удаление файлов программы"
	@sudo rm -rf ${SETTINGS_PATH} ${DATABASE_PATH}
	@sudo rm -f /usr/bin/learned-cat
	@sudo rm -f /usr/bin/learned-cat-daemon
	@sudo userdel -f learned-cat || echo "Пользователь learned-cat не был создан"
	@sudo groupdel learned-cat || echo "Гуппа learned-cat не была создана"
	@sudo rm -f /etc/bash_completion.d/learned-cat-daemon /etc/bash_completion.d/learned-cat

#@if [[ "systemd" = `ps --no-headers -o comm 1` ]]; then \
#	systemctl stop learned-cat; \
#	rm -f /etc/systemd/system/learned-cat.service; \
#	systemctl daemon-reload \
#fi;

all:
	cargo build --release
	cp target/release/learned-cat learned-cat
	cp target/release/learned-cat-daemon learned-cat-daemon

install: uninstall all
	sudo cp -r example-config /opt/learned-cat
	sudo cp learned-cat /usr/bin/learned-cat
	sudo cp learned-cat-daemon /opt/learned-cat/
	sudo useradd learnedcat || echo "Пользователь уже существует"
	sudo chown learnedcat:learnedcat /opt/learned-cat
	sudo chmod 770 /opt/learned-cat

	@if [[ "systemd" = `ps --no-headers -o comm 1` ]]; then \
	   sudo cp learned-cat.service /etc/systemd/system/learned-cat.service; \
	else \
		echo "ПРЕДУПРЕЖДЕНИЕ: Systemd не обнаружена. Используйте свой менеджер инициализации."; \
		echo "                Для запуска демона используйте команду /opt/learned-cat/learned-cat-daemon -r"; \
		echo "                Администратор демона: learnedcat"; \
	fi;

uninstall:
	@if [[ "systemd" = `ps --no-headers -o comm 1` ]]; then \
		sudo systemctl stop learned-cat; \
		sudo rm -f /etc/systemd/system/learned-cat.service; \
	fi;
	sudo rm -rf /opt/learned-cat
	sudo rm -f /usr/bin/learned-cat
	sudo userdel -f learnedcat || echo "Пользователь learnedcat не был создан"

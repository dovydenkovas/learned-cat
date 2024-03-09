#!/usr/bin/python
from random import shuffle
import re
import json
import os
import socket
import signal

# FIXME: Установить в соответствии с настройками сервера
SERVER_IP = '127.0.0.1'
SERVER_PORT = 65431


def main():
    signal.signal(signal.SIGTSTP, handlerCZ)
    if check_already_done():
        print('Вы уже выполнили тест. Спасибо!')
    else:
   	    play_test()


def no_error(func):
    def wrapper(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except Exception as e:
            print("Что-то пошло не так.")
            print(e) 
    return wrapper


@no_error
def play_test():
    '''  Основная функция теста. 
    Загружает вопросы, задает их, отправляет ответы. 
    '''
    print(get_banner())
    try:
        input('Нажмите Enter чтобы начать тест.')
    except KeyboardInterrupt:
        print()
        exit(0)

    questions = get_variant()
    for question in questions:
        answer = play_question(question)
        check_answer(answer)

    print('Тест завершен')
    n_true_answers, total_time = end_test()
    
    if n_true_answers == -1:
        print('К сожалению баллов недостаточно, попробуйте снова.')
        return
    
    print('{} из {} за {} секунд.'.format(
        n_true_answers, len(questions), total_time))


@no_error
def play_question(question):
    ''' Задает один вопрос. Возвращает ответ пользователя. '''
    
    print('\n——————————————————————————————————————————————————')
    print(question['question'])
    shuffle(question['answers'])
    for i in range(len(question['answers'])):
        print(str(i + 1) + '. ', question['answers'][i]['text'])
    print()
    answer = read_answer(len(question['answers']))
    result = []
    for i in answer:
        result.append(question['answers'][i]['id'])
    return {
        'id': question['id'],
        'answer': result }


@no_error
def read_answer(n_answers):
    ''' Считывает ответ, проверяет его корректность. '''
    
    while True:
        try:
            answer = input("Ваш ответ: ")

            answers = re.split('; |, | +', answer)
            result = set()
            for a in answers:
                if not a.isdigit():
                    print("Ответ должен состоять из чисел, записанных через пробел.")
                    break
                elif int(a) > n_answers or int(a) < 1:
                    print("Ответы могут быть от 1 до {}".format(n_answers))
                    break
                else:
                    result.add(int(a)-1)
            else:
                return tuple(result)
        except KeyboardInterrupt:
            print("\nНельзя прерывать тест.")
        except EOFError:
            print("\nНельзя прервать тест.")


@no_error
def check_already_done():
    return call_server({
        'command': 'check_done',
        'user': os.getlogin() }) 


@no_error
def get_banner():
    return call_server({'command': 'get_banner' })


@no_error
def get_variant(): 
    return call_server({'command': 'get_variant', 
                        'user': os.getlogin() })


@no_error
def check_answer(answer):
    return call_server({
        'command': 'check_answer',
        'answer': answer,
        'user': os.getlogin() })


@no_error
def end_test():
    return call_server({
        'command': 'end_test',
        'user': os.getlogin() })


def call_server(message):
    print("> ", message)
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        try:
            s.connect((SERVER_IP, SERVER_PORT))
            s.sendall(json.dumps(message).encode())
            data = s.recv(1000000) # Баг, возможно переполнение
            ret = json.loads(data.decode())
            print("< ", ret)
            return ret
        except Exception as e:
            print('Ошибка генерации варианта. Обратитесь к организатору теста.')
            print(e)
            exit()


def handlerCZ(signum, frame):
    print('Вы нажали Ctrl+Z. \nИспользуйте команду fg чтобы вернуться к тесту.')


if __name__ == '__main__':
    main()



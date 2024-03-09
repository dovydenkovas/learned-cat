create table Student (
    student_id SERIAL PRIMARY KEY,
    login VARCHAR(50) NOT NULL,
    is_allowed BOOLEAN
);


create table Test (
    test_id SERIAL PRIMARY KEY,
    caption VARCHAR(50) NOT NULL
);


create table Result (
    result_id SERIAL PRIMARY KEY,
    test_id INT,
    student_id INT,
    result INT,
    timestamp_start TIMESTAMP,
    timestamp_end TIMESTAMP
);

create table Answer (
    answer_id SERIAL PRIMARY KEY,
    result_id INT,
    question_id INT,
    answers INT ARRAY[20]
);


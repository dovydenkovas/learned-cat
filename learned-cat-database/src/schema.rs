// @generated automatically by Diesel CLI.

diesel::table! {
    Answer (id) {
        id -> Integer,
        question_id -> Integer,
        answer -> Text,
        is_correct -> Bool,
    }
}

diesel::table! {
    Question (id) {
        id -> Integer,
        test_id -> Integer,
        question -> Text,
    }
}

diesel::table! {
    Test (id) {
        id -> Integer,
        caption -> Text,
        description -> Text,
    }
}

diesel::table! {
    User (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    UserTest (id) {
        id -> Integer,
        user_id -> Integer,
        test_id -> Integer,
    }
}

diesel::table! {
    Variant (id) {
        id -> Integer,
        test_id -> Integer,
        user_id -> Integer,
        mark -> Float,
        begin_timestamp -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    Answer,
    Question,
    Test,
    User,
    UserTest,
    Variant,
);

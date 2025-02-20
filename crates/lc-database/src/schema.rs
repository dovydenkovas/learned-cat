// @generated automatically by Diesel CLI.

diesel::table! {
    tests (id) {
        id -> Integer,
        caption -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    variants (id) {
        id -> Integer,
        test_id -> Integer,
        user_id -> Integer,
        mark -> Float,
        start_timestamp -> Text,
        end_timestamp -> Text,
    }
}

diesel::table! {
    questions (id) {
        id -> Integer,
        variant_id -> Integer,
        text -> Text,
    }
}

diesel::table! {
    answers {
        id -> Integer,
        question_id -> Integer,
        text -> Text,
        is_correct -> Bool,
        is_selected -> Bool,
    }
}

diesel::joinable!(variants -> users (user_id));
diesel::joinable!(variants -> tests (test_id));
diesel::joinable!(questions -> variants (variant_id));
diesel::joinable!(answers -> questions (question_id));

diesel::allow_tables_to_appear_in_same_query!(tests, users, variants, questions, answers);

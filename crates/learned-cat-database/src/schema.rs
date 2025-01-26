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

diesel::joinable!(variants -> users (user_id));
diesel::joinable!(variants -> tests (test_id));

diesel::allow_tables_to_appear_in_same_query!(tests, users, variants,);

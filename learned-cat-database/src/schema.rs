#![allow(non_snake_case)]
// @generated automatically by Diesel CLI.

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
    Variant (id) {
        id -> Integer,
        test_id -> Integer,
        user_id -> Integer,
        mark -> Float,
        begin_timestamp -> Text,
    }
}

diesel::joinable!(Variant -> User (user_id));
diesel::joinable!(Variant -> Test (test_id));

diesel::allow_tables_to_appear_in_same_query!(Test, User, Variant,);

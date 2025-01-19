#![allow(non_snake_case)]
// @generated automatically by Diesel CLI.

use Variant::{end_timestamp, start_timestamp};

diesel::table! {
    Test (id) {
        id -> Integer,
        caption -> Text,
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
        test -> Integer,
        user -> Integer,
        mark -> Float,
        start_timestamp -> Text,
        end_timestamp -> Text,
    }
}

diesel::joinable!(Variant -> User (user));
diesel::joinable!(Variant -> Test (test));

diesel::allow_tables_to_appear_in_same_query!(Test, User, Variant,);

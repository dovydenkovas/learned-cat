use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::User)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::UserTest)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserTest {
    pub id: i32,
    pub user_id: i32,
    pub test_id: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::Test)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Test {
    pub id: i32,
    pub caption: String,
    pub description: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::Question)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Question {
    id: i32,
    test_id: i32,
    question: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::Answer)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Answer {
    id: i32,
    question_id: i32,
    answer: String,
    is_correct: bool,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::Variant)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Variant {
    id: i32,
    test_id: i32,
    user_id: i32,
    mark: f32,
    begin_timestamp: String,
}

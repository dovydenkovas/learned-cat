use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Selectable, Insertable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = crate::schema::tests)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Test {
    pub id: i32,
    pub caption: String,
}

#[derive(Queryable, Selectable, Insertable, Associations, Identifiable, Debug, PartialEq)]
#[diesel(table_name = crate::schema::variants)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Test))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Variant {
    pub id: i32,
    pub test_id: i32,
    pub user_id: i32,
    pub mark: f32,
    pub start_timestamp: String,
    pub end_timestamp: String,
}

#[derive(Queryable, Selectable, Insertable, Associations, Identifiable, Debug, PartialEq)]
#[diesel(table_name = crate::schema::questions)]
#[diesel(belongs_to(Variant))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Question {
    pub id: i32,
    pub variant_id: i32,
    pub text: String,
}

#[derive(Queryable, Selectable, Insertable, Associations, Identifiable, Debug, PartialEq)]
#[diesel(table_name = crate::schema::answers)]
#[diesel(belongs_to(Question))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Answer {
    pub id: i32,
    pub question_id: i32,
    pub text: String,
    pub is_correct: bool,
    pub is_selected: bool,
}

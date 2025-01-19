use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::User)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::Test)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Test {
    pub id: i32,
    pub caption: String,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::Variant)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Variant {
    pub id: i32,
    pub test: i32,
    pub user: i32,
    pub mark: f32,
    pub start_timestamp: String,
    pub end_timestamp: String,
}

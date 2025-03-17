use axum::http::StatusCode;
use chrono::NaiveDateTime;
use uuid::Uuid;

pub struct User {
    id: Uuid,
    name: String,
    email: String,
    status: u16,
    code: String,
    phone: String,
    created_time: NaiveDateTime,
    updated_time: NaiveDateTime,
    password: String,
}

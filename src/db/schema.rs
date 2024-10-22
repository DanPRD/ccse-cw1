// @generated automatically by Diesel CLI.

diesel::table! {
    sessions (id) {
        #[max_length = 255]
        id -> Varchar,
        user_id -> Integer,
        expires_at -> Datetime,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        is_admin -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    sessions,
    users,
);

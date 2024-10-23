// @generated automatically by Diesel CLI.

diesel::table! {
    products (id) {
        id -> Integer,
        #[max_length = 255]
        title -> Varchar,
        #[max_length = 255]
        description -> Varchar,
        #[max_length = 255]
        imgname -> Varchar,
    }
}

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
    products,
    sessions,
    users,
);

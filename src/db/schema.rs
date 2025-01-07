// @generated automatically by Diesel CLI.

diesel::table! {
    addresses (id) {
        id -> Integer,
        user_id -> Integer,
        #[max_length = 255]
        recipient_name -> Varchar,
        #[max_length = 255]
        line_1 -> Varchar,
        #[max_length = 255]
        line_2 -> Varchar,
        #[max_length = 8]
        postcode -> Varchar,
        #[max_length = 255]
        county -> Varchar,
    }
}

diesel::table! {
    cartproducts (product_id, user_id) {
        user_id -> Integer,
        product_id -> Integer,
        quantity -> Integer,
    }
}

diesel::table! {
    likedproducts (product_id, user_id) {
        user_id -> Integer,
        product_id -> Integer,
    }
}

diesel::table! {
    orders (id) {
        id -> Integer,
        user_id -> Integer,
        address_id -> Integer,
    }
}

diesel::table! {
    productorders (product_id, order_id) {
        product_id -> Integer,
        order_id -> Integer,
        quantity -> Integer,
    }
}

diesel::table! {
    products (id) {
        id -> Integer,
        #[max_length = 255]
        title -> Varchar,
        #[max_length = 255]
        description -> Varchar,
        #[max_length = 255]
        imgname -> Varchar,
        cost -> Decimal,
        listed -> Bool,
    }
}

diesel::table! {
    sessions (id) {
        #[max_length = 255]
        id -> Varchar,
        user_id -> Integer,
        expires_at -> Timestamptz,
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

diesel::joinable!(addresses -> users (user_id));
diesel::joinable!(cartproducts -> products (product_id));
diesel::joinable!(cartproducts -> users (user_id));
diesel::joinable!(likedproducts -> products (product_id));
diesel::joinable!(likedproducts -> users (user_id));
diesel::joinable!(orders -> addresses (address_id));
diesel::joinable!(orders -> users (user_id));
diesel::joinable!(productorders -> orders (order_id));
diesel::joinable!(productorders -> products (product_id));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    addresses,
    cartproducts,
    likedproducts,
    orders,
    productorders,
    products,
    sessions,
    users,
);

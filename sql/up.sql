CREATE TABLE users (
  id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  email VARCHAR(255) NOT NULL,
  password VARCHAR(255) NOT NULL,
  is_admin BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE sessions (
    id VARCHAR(255) PRIMARY KEY,
    user_id INT NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) 
);

CREATE TABLE products (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    title VARCHAR(255) NOT NULL,
    description VARCHAR(255) NOT NULL,
    imgname VARCHAR(255) NOT NULL,
    cost DECIMAL(4, 2) NOT NULL
);

CREATE TABLE addresses (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id INTEGER NOT NULL,
    recipient_name VARCHAR(255) NOT NULL,
    line_1 VARCHAR(255) NOT NULL,
    line_2 VARCHAR(255) NOT NULL,
    postcode VARCHAR(8) NOT NULL,
    county VARCHAR(255) NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);


CREATE TABLE orders (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id INTEGER NOT NULL,
    address_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (address_id) REFERENCES addresses(id)
);

CREATE TABLE productorders (
    product_id INTEGER NOT NULL,
    order_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    FOREIGN KEY (product_id) REFERENCES products(id),
    FOREIGN KEY (order_id) REFERENCES orders(id),
    PRIMARY KEY (product_id, order_id)
);

CREATE TABLE cartproducts (
    user_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (product_id) REFERENCES products(id),
    PRIMARY KEY (product_id, user_id)
);

CREATE TABLE likedproducts (
    user_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (product_id) REFERENCES products(id),
    PRIMARY KEY (product_id, user_id)
);

--user for unit and integration testing
INSERT INTO users VALUES(DEFAULT, 'testemail@securecart.com', '$argon2id$v=19$m=19456,t=2,p=1$xuZYri28ZUljWt1CvMXuwA$/j2hNwrsniZslvru/Te4CgOQb80/D9qwg28ZG64CLRM', FALSE);
CREATE TABLE users (
  id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  email VARCHAR(255) NOT NULL,
  password VARCHAR(255) NOT NULL,
  is_admin BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE sessions (
    id VARCHAR(255) PRIMARY KEY,
    user_id INT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) 
);

CREATE TABLE products (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    title VARCHAR(255) NOT NULL,
    description VARCHAR(255) NOT NULL,
    imgname VARCHAR(255) NOT NULL,
    cost DECIMAL(4, 2) NOT NULL,
    listed BOOLEAN NOT NULL DEFAULT TRUE
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
--default admin user, pass=@8*aUxB2#fEnT]E
INSERT INTO users VALUES(DEFAULT, 'admin@securecart.com', '$argon2id$v=19$m=19456,t=2,p=1$Rh8lGJODGahQiqlyvR48/Q$gNzg7gIWtjEI6pFnrgh1ZWxMxuS/xfGmvlEI/sSPRns', TRUE);

INSERT INTO products VALUES 
(DEFAULT,'Cinnamon Scented Candle','A candle that gives that warm smell to all those around it, a perfect candle for the autumn season','cinnamon.jpg', 12.50, DEFAULT),
(DEFAULT,'Cherry Scented Candle','A candle that gives a distinct cherry scent, like it was straight from the tree','cherry.jpg', 14.50, DEFAULT),
(DEFAULT,'Blackberry Scented Candle','Blackberry is a popular scent, giving a strong berry smell to fill the room','blackberry.jpg', 12, DEFAULT),
(DEFAULT,'Citrus Scented Candle','A strong orange and lemon smell, this candle can freshen up any room it is lit in','citrus.jpg', 10.50, DEFAULT),
(DEFAULT,'Coffee Scented Candle','Straight from coffee beans, lighting this in the morning is just like drinking a fresh cup of coffee!','coffee.jpg', 15, DEFAULT),
(DEFAULT,'Dahlia Scented Candle','A scented candle filled with the smell of dahlias, just like it came straight from the garden centre','dahlia.jpg', 14.50, DEFAULT),
(DEFAULT,'Floral Scented Candle','Bring the outside inside with this floral candle, which freshens any room that it is used in','floral.jpg', 10, DEFAULT),
(DEFAULT,'Lavender Scented Candle','A smell from the forest that is sure to bring a nice strong countryside smell to those near it','lavender.jpg', 14, DEFAULT),
(DEFAULT,'Ocean Scented Candle','Bring the seaside to your home with this ocean candle, with salty shores and bright sunets it is sure not to dissapoint','ocean.jpg', 15, DEFAULT),
(DEFAULT,'Peach Scented Candle','A fruity smell, it is sure to bring the tropical envrionment to yourdoorstep with this relaxing candle','peach.jpg', 11.50, DEFAULT),
(DEFAULT,'Pineapple Scented Candle','A tangy and sharp smell, the pineapple candle is sure to make a point in any area that its used in','pineapple.jpg', 16, DEFAULT),
(DEFAULT,'Pumpkin Scented Candle','A popular scent from the autumn season, the pumpkin candle is a definite pick for the spooky season bringing a warm atmosphere with it','pumpkin.jpg', 12.50, DEFAULT),
(DEFAULT,'Raspberry Scented Candle','A summer smell sure to brighten the day, anyone using this candle is sure to feel happier near it!','raspberry.jpg', 13.50, DEFAULT);

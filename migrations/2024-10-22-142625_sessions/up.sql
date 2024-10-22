-- Your SQL goes here
CREATE TABLE sessions (
    id VARCHAR(255) NOT NULL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id),
    expires_at DATETIME NOT NULL
)
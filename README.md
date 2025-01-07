E-Commerce web application for the CCSE Module focusing on security

Tech Stack:
- [Axum](https://github.com/tokio-rs/axum)
- [Askama](https://github.com/djc/askama)
- [Tailwind](https://tailwindcss.com/)
- [htmx](https://htmx.org/)

## prerequisites
1. Rust must be installed on your system. [link](https://www.rust-lang.org/learn/get-started)
2. PostgreSQL must be installed on your system. [link](https://www.postgresql.org/download/)

## Installation

1. Clone the repo into new directory
2. Create a new postgreSQL database, aswell as a new user with password protection. This account shoud only be able to access this new server
3. create a .env file in the project root, and enter the following:
```
DATABASE_URL=postgresql://username:password@localhost/database_name
```
where username, password and database_name are to be replaced with your own
4. execute the SQL file at `sql/up.sql`, then `sql/products.sql` to generate the correct tables and default entries
5. Build the project: 
```powershell
cargo build --release
```
6. Run the project:
```powershell
cargo run --release
```

The default admin account is as follows:
username: `admin@securecart.com`
password: `@8*aUxB2#fEnT]E`
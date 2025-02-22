# Architecture

## Web Server

### Frameworks
#### Axum
Axum is alright.  It runs on tokio but is relatively lightweight and has good extractors for defining routes.

### Endpoints
We will need endpoints for different data.
- Users
- Accounts/Balances as well as historical balances
- Institutions so we can connect our account to pull data from them

## Database
Treasury will use a PostgreSQL database to store data.  We will use sqlx to write our database repositories.

## Frontend
Not sure yet, but maybe a web assembly solution in Rust.

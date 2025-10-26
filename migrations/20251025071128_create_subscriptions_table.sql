-- Add migration script here
-- Create Subscriptions Table
CREATE TABLE subscriptions (
    id uuid NOT NULL PRIMARY KEY,
    email varchar(255) NOT NULL UNIQUE,
    name varchar(63) NOT NULL,
    subscribed_at timestamptz NOT NULL
);

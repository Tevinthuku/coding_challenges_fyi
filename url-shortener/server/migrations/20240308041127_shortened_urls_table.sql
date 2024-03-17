-- Add migration script here
CREATE TABLE urls (
    -- The default length will be 7, but we are just leaving enough space 
    -- in case of collisions and we need to increase the length of the colliding hash.
    hash VARCHAR(30) PRIMARY KEY,
    long_url TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
)

-- Add migration script here

CREATE UNIQUE INDEX long_url_idx ON urls (long_url) INCLUDE (hash);


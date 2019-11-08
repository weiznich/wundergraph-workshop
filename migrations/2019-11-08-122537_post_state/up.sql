-- Your SQL goes here

CREATE TYPE post_state AS ENUM ('Draft', 'Published', 'Deleted');

ALTER TABLE posts ADD COLUMN post_state post_state NOT NULL DEFAULT 'Published';

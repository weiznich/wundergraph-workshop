-- This file should undo anything in `up.sql`

ALTER TABLE posts DROP COLUMN post_state;
DROP TYPE post_state;

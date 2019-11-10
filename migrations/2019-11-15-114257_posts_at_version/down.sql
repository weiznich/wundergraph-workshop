-- This file should undo anything in `up.sql`

ALTER TABLE posts DROP CONSTRAINT posts_pkey;
ALTER TABLE posts ADD PRIMARY KEY (id);
ALTER TABLE comments ADD FOREIGN KEY (post) REFERENCES posts(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE posts DROP COLUMN version_start;
ALTER TABLE posts DROP COLUMN version_end;

DROP FUNCTION posts_at_version;

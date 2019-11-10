-- Your SQL goes here

ALTER TABLE posts ADD COLUMN version_start INTEGER NOT NULL DEFAULT 0;
ALTER TABLE posts ADD COLUMN version_end INTEGER DEFAULT NULL;
ALTER TABLE comments DROP CONSTRAINT comments_post_fkey;
ALTER TABLE posts DROP CONSTRAINT posts_pkey;
ALTER TABLE posts ADD PRIMARY KEY (id, version_start);

CREATE OR REPLACE FUNCTION posts_at_version (version int DEFAULT NULL)
RETURNS TABLE(id Integer, title Text, content Text, published_at Timestamp with time zone, author Integer, post_state post_state) AS $$
DECLARE
result record;
BEGIN
IF version IS NULL THEN
	RETURN QUERY SELECT posts.id, posts.title, posts.content, posts.published_at, posts.author, posts.post_state FROM posts WHERE projects.version_end IS NULL;
ELSE
	RETURN QUERY SELECT posts.id, posts.title, posts.content, posts.published_at, posts.author, posts.post_state FROM posts WHERE int4range(version_start, version_end, '[)') @> version;
END IF;
END;
$$ LANGUAGE plpgsql;

\set ECHO none
SET client_min_messages TO error;
SET synchronous_commit TO on;
\t on
\x off

SELECT 'created logical replication slot'
  FROM pg_create_logical_replication_slot('jsoncdc', 'jsoncdc');

BEGIN;

CREATE SCHEMA IF NOT EXISTS jsoncdc;
SET LOCAL search_path TO jsoncdc, public;

CREATE VIEW changedata AS
SELECT data FROM pg_logical_slot_get_changes('jsoncdc', NULL, NULL);

END;

SET search_path TO jsoncdc, public;

CREATE SEQUENCE xpto_rand_seq START 79 INCREMENT 1499; -- portable "random"
CREATE TABLE xpto (
    id serial PRIMARY KEY,
    toasted_col1 text,
    rand1 float8 DEFAULT nextval('xpto_rand_seq'),
    toasted_col2 text,
    rand2 float8 DEFAULT nextval('xpto_rand_seq')
);

--- Data that is TOAST but not so large as to be compressed.
INSERT INTO xpto (toasted_col1, toasted_col2)
SELECT string_agg(g.i::text, ''), string_agg((g.i*2)::text, '')
  FROM generate_series(1, 2000) g(i);

SELECT * FROM jsoncdc.changedata
 WHERE NOT (data LIKE '{ "begin": %' OR data LIKE '{ "commit": %');

--- Data large enough to force compression.
INSERT INTO xpto (toasted_col2)
SELECT repeat(string_agg(to_char(g.i, 'FM0000'), ''), 50)
  FROM generate_series(1, 500) g(i);

UPDATE xpto
   SET toasted_col1 = (SELECT string_agg(g.i::text, '')
                         FROM generate_series(1, 2000) AS g(i))
 WHERE id = 1;

DELETE FROM xpto WHERE id = 1;

SELECT * FROM jsoncdc.changedata
 WHERE NOT (data LIKE '{ "begin": %' OR data LIKE '{ "commit": %');

SELECT 'deleted logical replication slot'
  FROM pg_drop_replication_slot('jsoncdc');

DROP SCHEMA jsoncdc CASCADE;

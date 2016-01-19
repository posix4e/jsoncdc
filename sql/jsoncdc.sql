/*
 * Author: The maintainer's name
 * Created at: 2016-01-17 21:35:33 -0800
 *
 */

--
-- This is a example code genereted automaticaly
-- by pgxn-utils.

-- This is how you define a C function in PostgreSQL.
CREATE OR REPLACE FUNCTION jsoncdc(text)
RETURNS text
AS 'jsoncdc'
LANGUAGE C IMMUTABLE STRICT;

-- See more: http://www.postgresql.org/docs/current/static/xfunc-c.html

\set ECHO 0
BEGIN;
\i sql/jsoncdc.sql
\set ECHO all

-- You should write your tests

SELECT jsoncdc('test');

ROLLBACK;

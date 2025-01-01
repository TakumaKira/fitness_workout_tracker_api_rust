-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS users_id_trigger ON users;
DROP FUNCTION IF EXISTS users_id_handler();
DROP TABLE IF EXISTS users;
DROP SEQUENCE IF EXISTS users_id_seq;

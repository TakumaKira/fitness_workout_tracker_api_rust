-- Drop trigger first
DROP TRIGGER IF EXISTS sessions_id_trigger ON sessions;

-- Drop trigger function
DROP FUNCTION IF EXISTS sessions_id_handler();

-- Drop table (cascade will handle the foreign key constraint)
DROP TABLE IF EXISTS sessions; 
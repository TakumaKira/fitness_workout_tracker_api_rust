-- Drop trigger first
DROP TRIGGER IF EXISTS temp_sessions_id_trigger ON temp_sessions;

-- Drop trigger function
DROP FUNCTION IF EXISTS temp_sessions_id_handler();

-- Drop table
DROP TABLE IF EXISTS temp_sessions; 
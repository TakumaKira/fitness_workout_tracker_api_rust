DROP TRIGGER IF EXISTS update_workouts_updated_at ON workouts;
DROP FUNCTION IF EXISTS update_updated_at_column();
DROP TRIGGER IF EXISTS workouts_id_trigger ON workouts;
DROP FUNCTION IF EXISTS workouts_id_handler();
DROP TABLE IF EXISTS workouts; 
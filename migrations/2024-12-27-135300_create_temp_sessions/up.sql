CREATE TABLE "temp_sessions" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "session_id" VARCHAR NOT NULL UNIQUE,
    "csrf_token" VARCHAR NOT NULL,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "expires_at" TIMESTAMP NOT NULL
);

-- Create trigger for auto-incrementing ID
CREATE OR REPLACE FUNCTION temp_sessions_id_handler()
RETURNS TRIGGER AS $$
DECLARE
    next_id BIGINT;
BEGIN
    LOCK TABLE temp_sessions IN EXCLUSIVE MODE;
    SELECT COALESCE(MAX(id), 0) + 1 INTO next_id FROM temp_sessions;
    NEW.id := next_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER temp_sessions_id_trigger
    BEFORE INSERT ON temp_sessions
    FOR EACH ROW
    EXECUTE FUNCTION temp_sessions_id_handler(); 
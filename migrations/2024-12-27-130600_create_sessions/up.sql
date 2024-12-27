CREATE TABLE "sessions" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "user_id" BIGINT NOT NULL REFERENCES users(id),
    "token" VARCHAR NOT NULL UNIQUE,
    "csrf_token" VARCHAR NOT NULL,
    "expires_at" TIMESTAMP NOT NULL,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_user
        FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);

-- Create trigger for auto-incrementing ID
CREATE OR REPLACE FUNCTION sessions_id_handler()
RETURNS TRIGGER AS $$
DECLARE
    next_id BIGINT;
BEGIN
    LOCK TABLE sessions IN EXCLUSIVE MODE;
    SELECT COALESCE(MAX(id), 0) + 1 INTO next_id FROM sessions;
    NEW.id := next_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER sessions_id_trigger
    BEFORE INSERT ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION sessions_id_handler(); 
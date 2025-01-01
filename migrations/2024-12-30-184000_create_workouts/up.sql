CREATE TABLE "workouts" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "uuid" UUID NOT NULL UNIQUE,
    "user_id" BIGINT NOT NULL,
    "name" VARCHAR NOT NULL,
    "description" TEXT,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_user
        FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);

-- Create trigger for auto-incrementing ID
CREATE OR REPLACE FUNCTION workouts_id_handler()
RETURNS TRIGGER AS $$
DECLARE
    next_id BIGINT;
BEGIN
    LOCK TABLE workouts IN EXCLUSIVE MODE;
    SELECT COALESCE(MAX(id), 0) + 1 INTO next_id FROM workouts;
    NEW.id := next_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER workouts_id_trigger
    BEFORE INSERT ON workouts
    FOR EACH ROW
    EXECUTE FUNCTION workouts_id_handler();

-- Create trigger for updating updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_workouts_updated_at
    BEFORE UPDATE ON workouts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column(); 
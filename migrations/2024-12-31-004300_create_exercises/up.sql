CREATE TABLE exercises (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid UUID NOT NULL UNIQUE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    description VARCHAR,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE SEQUENCE exercises_id_seq NO CYCLE;

CREATE OR REPLACE FUNCTION exercises_id_handler() 
RETURNS trigger AS $$
BEGIN
    IF NEW.id = 0 THEN
        NEW.id = nextval('exercises_id_seq');
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER exercises_id_trigger
    BEFORE INSERT ON exercises
    FOR EACH ROW
    EXECUTE FUNCTION exercises_id_handler();

CREATE TABLE workout_exercises (
    workout_id BIGINT NOT NULL REFERENCES workouts(id) ON DELETE CASCADE,
    exercise_id BIGINT NOT NULL REFERENCES exercises(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    "order" INTEGER NOT NULL,
    PRIMARY KEY (workout_id, exercise_id)
);

CREATE INDEX exercises_user_id_idx ON exercises(user_id);
CREATE INDEX workout_exercises_workout_id_idx ON workout_exercises(workout_id);
CREATE INDEX workout_exercises_exercise_id_idx ON workout_exercises(exercise_id); 
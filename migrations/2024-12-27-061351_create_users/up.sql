-- Create sequence with NO CYCLE option
CREATE SEQUENCE users_id_seq NO CYCLE;

CREATE TABLE "users"(
	"id" BIGINT NOT NULL PRIMARY KEY,
	"uuid" UUID NOT NULL,
	"email" VARCHAR NOT NULL UNIQUE,
	"password_hash" VARCHAR NOT NULL,
	"created_at" TIMESTAMP NOT NULL,
	"updated_at" TIMESTAMP NOT NULL
);

-- Create function to get next ID only on successful insert
CREATE OR REPLACE FUNCTION users_id_handler()
RETURNS TRIGGER AS $$
DECLARE
	next_id BIGINT;
BEGIN
	-- Lock the table to prevent concurrent inserts
	LOCK TABLE users IN EXCLUSIVE MODE;
	
	-- Check for duplicate email before getting next ID
	IF EXISTS (SELECT 1 FROM users WHERE email = NEW.email) THEN
		RAISE unique_violation USING MESSAGE = 'duplicate key value violates unique constraint "users_email_key"';
	END IF;
	
	-- Get next ID only if no conflicts
	SELECT COALESCE(MAX(id), 0) + 1 INTO next_id FROM users;
	NEW.id := next_id;
	
	RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to handle ID assignment
CREATE TRIGGER users_id_trigger
	BEFORE INSERT ON users
	FOR EACH ROW
	EXECUTE FUNCTION users_id_handler();


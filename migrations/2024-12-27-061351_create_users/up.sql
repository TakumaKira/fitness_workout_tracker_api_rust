-- Your SQL goes here
CREATE TABLE "users"(
	"id" SERIAL8 NOT NULL PRIMARY KEY,
	"uuid" UUID NOT NULL,
	"email" VARCHAR NOT NULL UNIQUE,
	"password_hash" VARCHAR NOT NULL,
	"created_at" TIMESTAMP NOT NULL,
	"updated_at" TIMESTAMP NOT NULL
);


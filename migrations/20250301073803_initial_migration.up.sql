-- Add up migration script here
CREATE TABLE "user" (
        id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        name VARCHAR(254) NOT NULL,
        email VARCHAR(254) NOT NULL UNIQUE
);

CREATE INDEX ix_user_name ON "user" USING btree (lower(name));
CREATE INDEX ix_user_email ON "user" USING btree (lower(name));

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
        NEW.updated_at = CURRENT_TIMESTAMP;
        RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_user_updated_at
        BEFORE UPDATE ON "user"
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();

-- Add up migration script here
CREATE TABLE institution (
        id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        name VARCHAR(254) NOT NULL UNIQUE
);

CREATE TRIGGER update_institution_updated_at
        BEFORE UPDATE ON institution
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();

CREATE table asset (
        id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        name VARCHAR(254) NOT NULL UNIQUE,
        symbol VARCHAR(8) NOT NULL UNIQUE
);

CREATE TRIGGER update_asset_updated_at
        BEFORE UPDATE ON asset
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();

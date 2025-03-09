CREATE TABLE account (
        id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        user_id UUID NOT NULL,
        institution_id UUID NOT NULL,
        name VARCHAR(254) NOT NULL,
        CONSTRAINT fk_account_user_id_user FOREIGN KEY (user_id) REFERENCES "user" (id),
        CONSTRAINT fk_account_institution_id_institution FOREIGN KEY (institution_id) REFERENCES institution (id)
);

CREATE TRIGGER update_account_updated_at
        BEFORE UPDATE ON account
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();

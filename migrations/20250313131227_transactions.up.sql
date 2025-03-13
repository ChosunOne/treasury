CREATE TABLE "transaction" (
        id BIGSERIAL PRIMARY KEY,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        posted_at TIMESTAMPTZ NOT NULL,
        account_id UUID NOT NULL,
        asset_id UUID NOT NULL,
        description TEXT,
        quantity BIGINT NOT NULL,
        CONSTRAINT fk_transaction_account_id_account FOREIGN KEY (account_id) REFERENCES account (id),
        CONSTRAINT fk_transaction_asset_id_asset FOREIGN KEY (asset_id) REFERENCES asset (id)
);

CREATE TRIGGER update_transaction_updated_at
        BEFORE UPDATE ON "transaction"
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();

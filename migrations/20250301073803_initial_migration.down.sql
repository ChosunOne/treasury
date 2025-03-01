-- Add down migration script here
DROP TRIGGER update_user_updated_at;
DROP FUNCTION update_user_updated_at_column;
DROP INDEX ix_user_email;
DROP INDEX ix_user_name;
DROP TABLE "user";

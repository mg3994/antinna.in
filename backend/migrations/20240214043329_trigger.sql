-- Add migration script here
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
RETURN NEW;
END; $$ LANGUAGE plpgsql;


-- Create a function that prevents updating the firebase_uid column
CREATE OR REPLACE FUNCTION lock_firebase_uid()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.firebase_uid IS DISTINCT FROM OLD.firebase_uid THEN
        RAISE EXCEPTION 'firebase_uid is immutable and cannot be changed after creation.';
END IF;
RETURN NEW;
END; $$ LANGUAGE plpgsql;




CREATE OR REPLACE FUNCTION enforce_soft_delete()
RETURNS TRIGGER AS $$
BEGIN
    -- If row was not deleted before, and now someone tries to delete it
    IF OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL THEN
        NEW.deleted_at := CURRENT_TIMESTAMP;
END IF;

    -- Prevent restoring (user cannot set deleted_at back to NULL)
    IF OLD.deleted_at IS NOT NULL AND NEW.deleted_at IS NULL THEN
        RAISE EXCEPTION 'Cannot restore a deleted account';
END IF;

RETURN NEW;
END;
$$ LANGUAGE plpgsql;
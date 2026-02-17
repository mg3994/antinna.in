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


CREATE OR REPLACE FUNCTION mark_embedding_dirty()
RETURNS TRIGGER AS $$
BEGIN
  -- Always update timestamp
  NEW.updated_at = CURRENT_TIMESTAMP;

  -- Mark embedding stale if profile-relevant fields changed
  IF NEW.display_name IS DISTINCT FROM OLD.display_name
     OR NEW.bio IS DISTINCT FROM OLD.bio
     OR NEW.gender IS DISTINCT FROM OLD.gender
     OR NEW.dob IS DISTINCT FROM OLD.dob
     OR NEW.avatar_url IS DISTINCT FROM OLD.avatar_url
  THEN
     NEW.embedding_dirty = TRUE;
END IF;

RETURN NEW;
END;
$$ LANGUAGE plpgsql;



CREATE OR REPLACE FUNCTION auto_increment_embedding_version()
RETURNS TRIGGER AS $$
BEGIN
  -- Update timestamp automatically
  NEW.updated_at = CURRENT_TIMESTAMP;

  -- Increment version if embedding actually changed
  IF NEW.embedding IS DISTINCT FROM OLD.embedding
     OR NEW.model_name IS DISTINCT FROM OLD.model_name
     THEN
     NEW.version = OLD.version + 1;
     NEW.generated_at = CURRENT_TIMESTAMP;
END IF;

RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE OR REPLACE FUNCTION soft_delete_user_profile_embeddings()
  RETURNS trigger AS $$
BEGIN
 UPDATE user_profile_embeddings
 SET deleted_at = CURRENT_TIMESTAMP
WHERE user_id = OLD.user_id
  AND deleted_at IS NULL; -- Only update if not already soft-deleted


RETURN NULL; -- cancel delete
END;
$$ LANGUAGE plpgsql;


CREATE OR REPLACE FUNCTION handle_identity_verification()
RETURNS TRIGGER AS $$
BEGIN
    -- 1. If we are INSERTING and a value was provided, or if we are UPDATING
    -- and trying to set verified_at for the first time:
    IF (TG_OP = 'INSERT' AND NEW.verified_at IS NOT NULL) OR
       (TG_OP = 'UPDATE' AND OLD.verified_at IS NULL AND NEW.verified_at IS NOT NULL) THEN
        NEW.verified_at = CURRENT_TIMESTAMP;

    -- 2. If verified_at was already set in the past, LOCK IT.
    -- Prevent it from being changed or set back to NULL.
    ELSIF TG_OP = 'UPDATE' AND OLD.verified_at IS NOT NULL THEN
        NEW.verified_at = OLD.verified_at;
END IF;

RETURN NEW;
END;
$$ LANGUAGE plpgsql;



CREATE OR REPLACE FUNCTION handle_session_revocation()
RETURNS TRIGGER AS $$
BEGIN
    -- If the session is being updated and a 'revoke' signal is detected,
    -- or if we just want to ensure revoked_at isn't manually set to a fake date.
    IF NEW.revoked_at IS NOT NULL AND OLD.revoked_at IS NULL THEN
        NEW.revoked_at = CURRENT_TIMESTAMP;
END IF;

    NEW.updated_at = CURRENT_TIMESTAMP;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;
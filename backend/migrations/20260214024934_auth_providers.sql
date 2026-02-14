-- 1. Ensure the Enum exists with Firebase-style naming (except emailLink)
DO $$ BEGIN
CREATE TYPE auth_provider AS ENUM ('password', 'email_link', 'google.com', 'apple.com', 'phone');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- A single user actually has two different types of UIDs.
--
-- The Local UID (The "Sub"): This is the one you see in the Firebase Console. It represents the person.
--
-- The Provider UID: This is the ID from the source (e.g., the specific ID Google or Apple assigned to them).

CREATE TABLE IF NOT EXISTS  auth_identities (
      id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
      user_id UUID REFERENCES users(id) ON DELETE CASCADE,
      provider auth_provider NOT NULL,
    -- This is the 'uid' from the original provider not firebase as ==>'sub' or 'uid' from the Firebase decoded token
      provider_uid VARCHAR(255) NOT NULL,
    -- The human-readable identifier (the actual email or phone number)
      identifier VARCHAR(255),
    -- Better than BOOLEAN: NULL = Not Verified, Date = Verified at...
      verified_at TIMESTAMP WITH TIME ZONE, -- will be Null till we don't make it , useful in case of email/password , <= send email and verify kind stuff
      created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
   -- Constraints (Commas added, these were missing)
      UNIQUE(user_id, provider),
      UNIQUE(provider, provider_uid)
);
-- 3. Dedicated Trigger for auto-updating updated_at
-- Using a unique name for this table's trigger
DROP TRIGGER IF EXISTS trg_auth_identities_updated_at ON auth_identities;
--
CREATE TRIGGER trg_auth_identities_updated_at
    BEFORE UPDATE ON auth_identities
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
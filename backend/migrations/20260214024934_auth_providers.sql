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
    user_id       UUID REFERENCES users(id) ON DELETE CASCADE,
    provider      auth_provider NOT NULL,
    provider_uid  VARCHAR(255) NOT NULL,
    identifier    VARCHAR(255),
    verified_at   TIMESTAMP WITH TIME ZONE,
    created_at    TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
   -- This replaces the 'id' column entirely
    PRIMARY KEY (provider, provider_uid),

    -- Ensures a user doesn't link the same provider twice (e.g., two Google accounts)
    CONSTRAINT uni_user_provider UNIQUE(user_id, provider)
);
-- 3. Dedicated Trigger for auto-updating updated_at
-- Using a unique name for this table's trigger
DROP TRIGGER IF EXISTS trg_auth_identities_updated_at ON auth_identities;
--
CREATE TRIGGER trg_auth_identities_updated_at
    BEFORE UPDATE ON auth_identities
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
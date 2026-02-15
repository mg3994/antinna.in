-- 1. Ensure the Enum exists with Firebase-style naming (except emailLink)
CREATE TABLE IF NOT EXISTS provider_types (
 slug TEXT PRIMARY KEY, -- 'google.com', 'apple.com', 'password'
 name TEXT NOT NULL,    -- 'Google', 'Apple', 'Email/Password'
 is_active BOOLEAN DEFAULT true
);

-- Seed the table with Firebase-standard slugs
INSERT INTO provider_types (slug, name) VALUES
 ('password', 'Email/Password'),
 ('email_link', 'Email Link'), --emailLink
 ('google.com', 'Google'),
 ('apple.com', 'Apple'),
 ('phone', 'Phone Number')
    ON CONFLICT DO NOTHING;
-- A single user actually has two different types of UIDs.
--
-- The Local UID (The "Sub"): This is the one you see in the Firebase Console. It represents the person.
--
-- The Provider UID: This is the ID from the source (e.g., the specific ID Google or Apple assigned to them).

CREATE TABLE IF NOT EXISTS  auth_identities (
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Foreign Key to our lookup table
    provider      TEXT NOT NULL REFERENCES provider_types(slug),
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


-- 1. Enable Row Level Security
ALTER TABLE auth_identities ENABLE ROW LEVEL SECURITY;

-- 2. Create the Policy
-- This ensures that for any SELECT, UPDATE, or DELETE,
-- the user_id in the row must match the UUID we set in the session.
CREATE POLICY auth_identities_owner_policy ON auth_identities
    USING (user_id = current_setting('app.current_user_id')::uuid)
    WITH CHECK (user_id = current_setting('app.current_user_id')::uuid);

-- Note: 'WITH CHECK' prevents a user from trying to INSERT or UPDATE
-- a row to belong to a different user_id.
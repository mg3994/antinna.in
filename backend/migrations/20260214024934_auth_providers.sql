-- 1. Ensure the Enum exists with Firebase-style naming (except emailLink)
CREATE TABLE IF NOT EXISTS provider_types (
 slug TEXT PRIMARY KEY, -- 'google.com', 'apple.com', 'password'
 name TEXT NOT NULL,    -- 'Google', 'Apple', 'Email Magic Link'
 is_active BOOLEAN DEFAULT true
);

-- Seed the table with Firebase-standard slugs
INSERT INTO provider_types (slug, name) VALUES
 ('password', 'Email Link'),
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
DROP TRIGGER IF EXISTS trg_handle_verification ON auth_identities;
--
CREATE TRIGGER trg_auth_identities_updated_at
    BEFORE UPDATE ON auth_identities
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_handle_verification
    BEFORE INSERT OR UPDATE ON auth_identities
    FOR EACH ROW
EXECUTE FUNCTION handle_identity_verification();

-- 1. Enable RLS
ALTER TABLE auth_identities ENABLE ROW LEVEL SECURITY;

-- Clean up existing policies for auth_identities
DROP POLICY IF EXISTS auth_identities_select_owner ON auth_identities;
DROP POLICY IF EXISTS auth_identities_insert_owner ON auth_identities;
DROP POLICY IF EXISTS auth_identities_update_owner ON auth_identities;
DROP POLICY IF EXISTS auth_identities_delete_owner ON auth_identities;

-- 2. SELECT: "I can see my own linked accounts"
CREATE POLICY auth_identities_select_owner ON auth_identities
    FOR SELECT
    USING (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);

-- 3. INSERT: "I can link a new provider to my account"
CREATE POLICY auth_identities_insert_owner ON auth_identities
    FOR INSERT
    WITH CHECK (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);

-- 4. UPDATE: "I can update my provider details (email, etc) but NOT the owner ID"
CREATE POLICY auth_identities_update_owner ON auth_identities
    FOR UPDATE
    USING (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid)
    WITH CHECK (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);

-- 5. DELETE: "I can unlink a provider from my account"
CREATE POLICY auth_identities_delete_owner ON auth_identities
    FOR DELETE
    USING (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);

-- add triger for verified at and make sure once verfied at is set then avoid seting up verified_At again
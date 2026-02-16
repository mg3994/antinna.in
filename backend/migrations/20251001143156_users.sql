CREATE TYPE gender_enum AS ENUM (
    'male',
    'female',
    'non_binary',
    'transgender',
    'intersex',
    'prefer_not_to_say',
    'other'
    );


-- A single user actually has two different types of UIDs.
--
-- The Local UID (The "Sub"): This is the one you see in the Firebase Console. It represents the person.
--
-- The Provider UID: This is the ID from the source (e.g., the specific ID Google or Apple assigned to them).
--
-- The "Single Identity" Reality
-- In Firebase, even if a user links Google and Email/Password, they usually have one single Firebase UID (the sub in the JWT).

-- 2. Create Users Table
CREATE TABLE IF NOT EXISTS users
(
    -- Unique internal ID, strictly NOT NULL
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    firebase_uid TEXT  NOT NULL UNIQUE,

    display_name VARCHAR(100),
    bio TEXT,
    avatar_url TEXT,
    gender gender_enum,     -- nullable enum
    dob DATE,               -- date of birth


    embedding_dirty BOOLEAN DEFAULT TRUE,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_firebase_uid ON users(firebase_uid);
CREATE INDEX IF NOT EXISTS idx_users_active
    ON users(id)
    WHERE deleted_at IS NULL;

-- 2. Unique Username Table
-- Enforces: One user = One username, and every username is unique.
CREATE TABLE IF NOT EXISTS usernames (
    -- Setting user_id as PRIMARY KEY ensures a user can only appear ONCE in this table.
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,

    -- UNIQUE constraint ensures no two users share the same handle.
    username CITEXT NOT NULL UNIQUE,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- 1. THE POLICY CONSTRAINT (Replaces your individual checks)
    CONSTRAINT username_policy CHECK (
        -- Length: 3 to 30 chars
        -- Regex: Must start with a letter, then can have letters, numbers, or single underscores.
        -- This automatically handles: lowercase, no spaces, and no weird symbols.
        username::text ~ '^[a-z][a-z0-9_]{2,29}$'

            -- Prevent "ugly" handles like 'john__doe'
            AND username::text !~ '__'

            -- Ensure it doesn't end with an underscore (looks better in URLs)
            AND username::text !~ '_$'
        ),

    -- 2. RESERVED WORDS (Crucial for Marketplace security)
    CONSTRAINT username_not_reserved CHECK (
        username::text NOT IN (
            'admin', 'support', 'official', 'system', 'root',
            'help', 'marketplace', 'billing', 'api', 'mentors'
            )
        )
);


-- This allows: SELECT * FROM usernames WHERE username % 'Zhan San'; (The % operator is fuzzy match)
-- Fast fuzzy suggestions
CREATE INDEX idx_usernames_trgm
    ON usernames USING gin (username gin_trgm_ops);

-- Fast prefix search
CREATE INDEX idx_username_prefix
    ON usernames (username text_pattern_ops);

CREATE TABLE IF NOT EXISTS user_profile_embeddings
(
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,

    embedding VECTOR(1536) NOT NULL,

    model_name VARCHAR(50),

    embedding_source TEXT, -- <- will make enum soon
    version INT DEFAULT 1,

    generated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE DEFAULT NULL
);

CREATE INDEX IF NOT EXISTS idx_user_profile_embeddings_vector
    ON user_profile_embeddings
USING ivfflat (embedding vector_cosine_ops);

-- 3.1. Drop if
DROP TRIGGER IF EXISTS trg_users_updated_at ON users;
DROP TRIGGER IF EXISTS trg_usernames_updated_at ON usernames;
DROP TRIGGER IF EXISTS trg_lock_firebase_uid ON users;
DROP TRIGGER IF EXISTS trg_enforce_soft_delete ON users;
DROP TRIGGER IF EXISTS trg_mark_embedding_dirty ON users;
DROP TRIGGER IF EXISTS trg_embedding_versioning ON user_profile_embeddings;

DROP TRIGGER IF EXISTS trg_soft_delete_user_profile_embeddings ON user_profile_embeddings;
-- 3.2. Attach Trigger
-- no need of updated at embedding => trg_mark_embedding_dirty will handle that
-- CREATE TRIGGER trg_users_updated_at
--     BEFORE UPDATE ON users
--     FOR EACH ROW
--     EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_usernames_updated_at
    BEFORE UPDATE ON usernames
    FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

-- Attach the trigger to the users table
CREATE TRIGGER trg_lock_firebase_uid
    BEFORE UPDATE ON users
    FOR EACH ROW
EXECUTE FUNCTION lock_firebase_uid();

CREATE TRIGGER trg_enforce_soft_delete
    BEFORE UPDATE ON users
    FOR EACH ROW
EXECUTE FUNCTION enforce_soft_delete();

CREATE TRIGGER trg_mark_embedding_dirty
    BEFORE UPDATE ON users
    FOR EACH ROW
EXECUTE FUNCTION mark_embedding_dirty();

CREATE TRIGGER trg_embedding_versioning
    BEFORE UPDATE ON user_profile_embeddings
    FOR EACH ROW
EXECUTE FUNCTION auto_increment_embedding_version();

CREATE TRIGGER trg_soft_delete_user_profile_embeddings
    BEFORE DELETE ON user_profile_embeddings
    FOR EACH ROW
EXECUTE FUNCTION soft_delete_user_profile_embeddings();


-- ---
-- 1. Users Table Policies
-- ---
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

ALTER TABLE usernames ENABLE ROW LEVEL SECURITY;

-- CREATE VIEW public_profiles AS
-- SELECT
--     id,
--     display_name,
--     avatar_url
-- FROM users
-- WHERE deleted_at IS NULL;

-- [READ] Public: Anyone can see profiles (TODO: a better logic in future , where a some a  user having provide some specific services can even access even with deleted_At is not Null all other type be with where deleted_at is NULL
CREATE POLICY users_select_public ON users
    FOR SELECT
    USING (true); --(deleted_at IS NULL); // let decide it by roles column if role has that much bit then only we will allow

-- [UPDATE] Private: Only the owner can change their bio/avatar
-- We use NULLIF to prevent errors if the session variable isn't set yet
CREATE POLICY users_update_owner ON users
    FOR UPDATE
    USING (id = NULLIF(current_setting('app.current_user_id', true), '')::uuid
    AND deleted_at IS NULL )
    WITH CHECK (id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);



-- Note: No INSERT policy. This means only a superuser or the Rust App
-- can create a user (before the RLS variable is set).

-- ---
-- 3. Usernames Table Policies
-- ---

-- [READ] Public: Anyone can look up a username
CREATE POLICY usernames_select_public ON usernames
    FOR SELECT
    USING (true);

-- [UPDATE] Private: Only the owner can change their specific username
CREATE POLICY usernames_update_owner ON usernames
    FOR UPDATE
    USING (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid)
    WITH CHECK (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);

CREATE POLICY users_delete_never
    ON users
    FOR DELETE
    USING (false);
-- [DELETE] Private: Only the owner can delete their username record --> no one can delete , just soft delete
-- CREATE POLICY usernames_delete_owner ON usernames
--     FOR DELETE
--     USING (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);

-- [INSERT] Private: A user can only create a username for THEMSELVES
-- This is used if the user creates a profile first, then chooses a username later
CREATE POLICY usernames_insert_owner ON usernames
    FOR INSERT
    WITH CHECK (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);
-- INSERT INTO users (id, username, password) VALUES ('cdd0e080-5bb1-4442-b6f7-2ba60dbd0555', 'zhangsan', '$argon2id$v=19$m=19456,t=2,p=1$rcosL5pOPdA2c7i4ZuLA4Q$s0JGh78UzMmu1qZMpVUA3b8kWYLXcZhw7uBfwhYDJ4A');


-- Production Only TODO: uncomment Below
-- REVOKE DELETE ON users FROM PUBLIC;
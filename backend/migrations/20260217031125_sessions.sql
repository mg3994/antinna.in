-- Add migration script here
CREATE TABLE IF NOT EXISTS users_sessions (
                                              id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    device_id TEXT NOT NULL,
    fcm_token TEXT,
    user_agent TEXT,
    ip_address TEXT,

    auth_exp TIMESTAMP WITH TIME ZONE NOT NULL,
    revoked_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(user_id, device_id)
    );

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON users_sessions(user_id);

-- FIXED: Removed CURRENT_TIMESTAMP from the partial index
CREATE INDEX IF NOT EXISTS idx_active_push_tokens ON users_sessions(fcm_token)
    WHERE (revoked_at IS NULL);

DROP TRIGGER IF EXISTS trg_users_sessions_mgmt ON users_sessions;

CREATE TRIGGER trg_users_sessions_mgmt
    BEFORE UPDATE ON users_sessions
    FOR EACH ROW
    EXECUTE FUNCTION handle_session_revocation();

ALTER TABLE users_sessions ENABLE ROW LEVEL SECURITY;

-- Allow internal selection for middleware validation
DROP POLICY IF EXISTS sessions_select_internal ON users_sessions;
CREATE POLICY sessions_select_internal ON users_sessions
    FOR SELECT
                        USING (true);

-- [INSERT]
DROP POLICY IF EXISTS sessions_insert_owner ON users_sessions;
CREATE POLICY sessions_insert_owner ON users_sessions
    FOR INSERT
    WITH CHECK (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);

-- [UPDATE]
DROP POLICY IF EXISTS sessions_update_owner ON users_sessions;
CREATE POLICY sessions_update_owner ON users_sessions
    FOR UPDATE
                                    USING (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid)
        WITH CHECK (user_id = NULLIF(current_setting('app.current_user_id', true), '')::uuid);

-- [DELETE]
DROP POLICY IF EXISTS sessions_delete_never ON users_sessions;
CREATE POLICY sessions_delete_never ON users_sessions
    FOR DELETE
USING (false);
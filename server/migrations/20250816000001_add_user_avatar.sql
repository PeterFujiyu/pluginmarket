-- Add avatar support to users table

-- Add avatar_url column to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(500);

-- Create user_avatars table for metadata and management
CREATE TABLE IF NOT EXISTS user_avatars (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    upload_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,
    UNIQUE(user_id, is_active) -- Only one active avatar per user
);

-- Create index for performance
CREATE INDEX IF NOT EXISTS idx_user_avatars_user_id ON user_avatars(user_id);
CREATE INDEX IF NOT EXISTS idx_user_avatars_upload_time ON user_avatars(upload_time);
CREATE INDEX IF NOT EXISTS idx_user_avatars_active ON user_avatars(user_id, is_active) WHERE is_active = true;

-- Add trigger to update users.avatar_url when user_avatars is updated
CREATE OR REPLACE FUNCTION update_user_avatar_url()
RETURNS TRIGGER AS $$
BEGIN
    -- Update avatar_url in users table when a new active avatar is added
    IF NEW.is_active = true THEN
        -- Deactivate other avatars for this user
        UPDATE user_avatars 
        SET is_active = false 
        WHERE user_id = NEW.user_id AND id != NEW.id;
        
        -- Update user's avatar_url
        UPDATE users 
        SET avatar_url = '/api/v1/avatars/' || NEW.file_name
        WHERE id = NEW.user_id;
    END IF;
    
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger for avatar updates
CREATE TRIGGER update_user_avatar_url_trigger 
    AFTER INSERT OR UPDATE ON user_avatars 
    FOR EACH ROW 
    EXECUTE FUNCTION update_user_avatar_url();

-- Add trigger to clean up avatar_url when avatar is deleted
CREATE OR REPLACE FUNCTION cleanup_user_avatar_url()
RETURNS TRIGGER AS $$
BEGIN
    -- If the deleted avatar was active, clear the user's avatar_url
    IF OLD.is_active = true THEN
        UPDATE users 
        SET avatar_url = NULL 
        WHERE id = OLD.user_id;
    END IF;
    
    RETURN OLD;
END;
$$ language 'plpgsql';

-- Create trigger for avatar deletion
CREATE TRIGGER cleanup_user_avatar_url_trigger 
    AFTER DELETE ON user_avatars 
    FOR EACH ROW 
    EXECUTE FUNCTION cleanup_user_avatar_url();
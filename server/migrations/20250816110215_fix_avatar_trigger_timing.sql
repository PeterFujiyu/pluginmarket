-- Fix avatar trigger timing to run before insert to avoid constraint violation

-- Drop existing trigger 
DROP TRIGGER IF EXISTS update_user_avatar_url_trigger ON user_avatars;

-- Recreate trigger function to handle BEFORE INSERT
CREATE OR REPLACE FUNCTION update_user_avatar_url()
RETURNS TRIGGER AS $$
BEGIN
    -- If inserting/updating an active avatar, deactivate others first
    IF NEW.is_active = true THEN
        -- Deactivate other avatars for this user
        UPDATE user_avatars 
        SET is_active = false 
        WHERE user_id = NEW.user_id AND id != COALESCE(NEW.id, -1);
        
        -- Update user's avatar_url (only on INSERT or when activating)
        UPDATE users 
        SET avatar_url = '/api/v1/avatars/' || NEW.file_name
        WHERE id = NEW.user_id;
    END IF;
    
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create BEFORE INSERT/UPDATE trigger
CREATE TRIGGER update_user_avatar_url_trigger 
    BEFORE INSERT OR UPDATE ON user_avatars 
    FOR EACH ROW 
    EXECUTE FUNCTION update_user_avatar_url();
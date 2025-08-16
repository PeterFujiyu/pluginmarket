-- Fix avatar unique constraint to allow multiple inactive avatars per user
-- but only one active avatar per user

-- Drop the problematic unique constraint if it exists
ALTER TABLE user_avatars DROP CONSTRAINT IF EXISTS user_avatars_user_id_is_active_key;

-- Create a proper partial unique index that only applies to active avatars
CREATE UNIQUE INDEX IF NOT EXISTS user_avatars_one_active_per_user ON user_avatars (user_id) WHERE is_active = true;

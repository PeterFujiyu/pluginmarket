-- Add is_active column to plugins table for easier management
-- Note: status field already exists with plugin_status enum, but add is_active for cleaner admin interface
ALTER TABLE plugins ADD COLUMN IF NOT EXISTS is_active BOOLEAN DEFAULT true;

-- Update is_active based on current status
UPDATE plugins SET is_active = (status = 'active');

-- Add index for faster queries on active plugins
CREATE INDEX IF NOT EXISTS idx_plugins_is_active ON plugins(is_active);

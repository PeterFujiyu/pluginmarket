-- Add ethereum_address field to users table for Web3 authentication
ALTER TABLE users
ADD COLUMN ethereum_address VARCHAR(42) UNIQUE;

-- Create index for efficient lookups
CREATE INDEX idx_users_ethereum_address ON users(ethereum_address);

-- Add comment for documentation
COMMENT ON COLUMN users.ethereum_address IS 'Ethereum wallet address for Web3 authentication (optional)';

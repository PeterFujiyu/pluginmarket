-- Create user account bindings table to manage email-wallet account binding
CREATE TABLE user_account_bindings (
    id SERIAL PRIMARY KEY,
    primary_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    secondary_user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    binding_type VARCHAR(50) NOT NULL, -- 'email_to_wallet', 'wallet_to_email'
    email VARCHAR(255),
    ethereum_address VARCHAR(42),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT true,
    
    -- Constraints to ensure data integrity
    CONSTRAINT unique_email_binding UNIQUE(email),
    CONSTRAINT unique_ethereum_binding UNIQUE(ethereum_address),
    CONSTRAINT valid_binding_type CHECK (binding_type IN ('email_to_wallet', 'wallet_to_email')),
    CONSTRAINT check_email_or_eth_address CHECK (
        (binding_type = 'email_to_wallet' AND ethereum_address IS NOT NULL) OR
        (binding_type = 'wallet_to_email' AND email IS NOT NULL)
    )
);

-- Create indexes for efficient lookups
CREATE INDEX idx_user_bindings_primary_user_id ON user_account_bindings(primary_user_id);
CREATE INDEX idx_user_bindings_secondary_user_id ON user_account_bindings(secondary_user_id);
CREATE INDEX idx_user_bindings_email ON user_account_bindings(email);
CREATE INDEX idx_user_bindings_ethereum_address ON user_account_bindings(ethereum_address);
CREATE INDEX idx_user_bindings_active ON user_account_bindings(is_active);

-- Add comments for documentation
COMMENT ON TABLE user_account_bindings IS 'Manages binding relationships between email and wallet accounts';
COMMENT ON COLUMN user_account_bindings.primary_user_id IS 'The main user account ID that other accounts bind to';
COMMENT ON COLUMN user_account_bindings.secondary_user_id IS 'The secondary user account ID that gets merged (optional)';
COMMENT ON COLUMN user_account_bindings.binding_type IS 'Type of binding: email_to_wallet or wallet_to_email';
COMMENT ON COLUMN user_account_bindings.email IS 'Email address being bound (for wallet_to_email bindings)';
COMMENT ON COLUMN user_account_bindings.ethereum_address IS 'Ethereum address being bound (for email_to_wallet bindings)';

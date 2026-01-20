-- Add msg_type and ttl columns to messages table
ALTER TABLE messages ADD COLUMN msg_type TEXT NOT NULL DEFAULT 'Direct';
ALTER TABLE messages ADD COLUMN ttl INTEGER;

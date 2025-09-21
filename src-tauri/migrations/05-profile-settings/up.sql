ALTER TABLE profiles
ADD COLUMN custom_args JSON;

ALTER TABLE profiles
ADD COLUMN custom_args_enabled BOOLEAN DEFAULT 0;
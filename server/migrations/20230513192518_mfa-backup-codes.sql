ALTER TABLE Users
ADD COLUMN mfa_backup_codes varchar(8)[] null;

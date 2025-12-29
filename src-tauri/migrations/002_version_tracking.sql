-- Add version tracking fields for simplified update detection
-- version_sort_key: Pre-computed integer from index for direct comparison
-- commit_sha: For branch-based version tracking

ALTER TABLE installed_addons ADD COLUMN version_sort_key INTEGER;
ALTER TABLE installed_addons ADD COLUMN commit_sha TEXT;

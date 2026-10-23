CREATE TABLE readme_cache (
    version_id UUID NOT NULL PRIMARY KEY,
    created_at TIMESTAMP NOT NULl DEFAULT CURRENT_TIMESTAMP,
    content BLOB
);

CREATE TABLE changelog_cache (
    version_id UUID NOT NULL PRIMARY KEY,
    created_at TIMESTAMP NOT NULl DEFAULT CURRENT_TIMESTAMP,
    content BLOB
);

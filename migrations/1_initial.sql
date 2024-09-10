CREATE TABLE communities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    override_path TEXT,
    is_favorite BOOLEAN NOT NULL DEFAULT 0
);

INSERT INTO communities (name, slug)
VALUES 
    ('Lethal Company', 'lethal-company'),
    ('Content Warning', 'content-warning');

CREATE TABLE packages (
    id UUID PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    owner TEXT NOT NULL,
    description TEXT NOT NULL,
    date_created DATETIME NOT NULL,
    has_nsfw_content BOOLEAN NOT NULL,
    is_deprecated BOOLEAN NOT NULL,
    is_pinned BOOLEAN NOT NULL,
    rating_score INTEGER NOT NULL,
    downloads INTEGER NOT NULL,
    donation_link TEXT,
    latest_version_id UUID NOT NULL,
    community_id INTEGER NOT NULL REFERENCES communities(id)
);

CREATE VIRTUAL TABLE packages_fts
USING fts5(package_id, name, description, owner);

CREATE TRIGGER IF NOT EXISTS insert_package_fts 
    AFTER INSERT ON packages
BEGIN
    INSERT INTO packages_fts(package_id, name, description, owner) 
    VALUES (NEW.id, NEW.name, NEW.description, NEW.owner);
END;

CREATE TRIGGER IF NOT EXISTS update_package_fts 
    AFTER UPDATE ON packages
BEGIN
    UPDATE packages_fts
    SET
        name = NEW.name,
        description = NEW.description,
        owner = NEW.owner
    WHERE package_id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS delete_package_fts 
    AFTER DELETE ON packages
BEGIN
    DELETE FROM packages_fts
    WHERE package_id = OLD.id;
END;

CREATE TABLE versions (
    id UUID NOT NULL PRIMARY KEY,
    package_id UUID NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    date_created DATETIME NOT NULL,
    downloads INTEGER NOT NULL,
    file_size INTEGER NOT NULL,
    is_active BOOLEAN NOT NULL,
    website_url TEXT,
    major INTEGER NOT NULL,
    minor INTEGER NOT NULL,
    patch INTEGER NOT NULL
);

CREATE TABLE dependencies (
    dependent_id UUID NOT NULL REFERENCES versions(id) ON DELETE CASCADE,
    owner TEXT NOT NULL,
    name TEXT NOT NULL,
    major INTEGER NOT NULL,
    minor INTEGER NOT NULL,
    patch INTEGER NOT NULL,
    PRIMARY KEY (dependent_id, owner, name)
);

CREATE TABLE profiles (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    community_id INTEGER NOT NULL REFERENCES communities(id) ON DELETE CASCADE
);

CREATE TABLE profile_mods (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL,
    order_index INTEGER NOT NULL,
    source BLOB NOT NULL
);

CREATE TABLE settings (
    zoom_level FLOAT NOT NULL DEFAULT 1.0,
    steam_executable_path TEXT,
    steam_library_path TEXT,
    cache_path TEXT NOT NULL
);

CREATE TABLE misc (
    selected_profile_id INTEGER REFERENCES profiles(id)
);

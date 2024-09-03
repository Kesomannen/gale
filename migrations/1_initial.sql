CREATE TABLE communities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL
);

INSERT INTO communities (name, slug)
VALUES 
    ('Lethal Company', 'lethal-company'),
    ('Content Warning', 'content-warning');

CREATE TABLE packages (
    id UUID PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    date_created DATETIME NOT NULL,
    date_updated DATETIME NOT NULL,
    donation_link TEXT,
    has_nsfw_content BOOLEAN NOT NULL,
    is_deprecated BOOLEAN NOT NULL,
    is_pinned BOOLEAN NOT NULL,
    owner TEXT NOT NULL,
    rating_score INT NOT NULL,
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
    id UUID PRIMARY KEY NOT NULL,
    package_id UUID NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    date_created DATETIME NOT NULL,
    description TEXT NOT NULL,
    downloads INT NOT NULL,
    file_size INT NOT NULL,
    full_name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL,
    name TEXT NOT NULL,
    website_url TEXT,
    major INT NOT NULL,
    minor INT NOT NULL,
    patch INT NOT NULL
);

CREATE TABLE profiles (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    game_id UUID NOT NULL REFERENCES packages(id) ON DELETE CASCADE
);

CREATE TABLE profile_mods (
    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    mod_id UUID NOT NULL REFERENCES versions(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL,
    PRIMARY KEY (profile_id, mod_id)
);

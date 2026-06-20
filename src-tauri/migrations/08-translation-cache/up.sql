CREATE TABLE translation_cache (
    mod_uuid TEXT NOT NULL,
    language TEXT NOT NULL,
    original_name TEXT NOT NULL,
    original_description TEXT,
    translated_name TEXT NOT NULL,
    translated_description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (mod_uuid, language)
);

CREATE INDEX idx_translation_cache_lookup ON translation_cache(mod_uuid, language);

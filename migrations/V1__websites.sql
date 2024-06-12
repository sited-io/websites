CREATE TABLE websites (
    website_id VARCHAR NOT NULL PRIMARY KEY,
    user_id VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() ON UPDATE NOW(),
    name VARCHAR NOT NULL,
    domain VARCHAR NOT NULL UNIQUE,
    client_id VARCHAR NOT NULL UNIQUE,
    zitadel_app_id VARCHAR NOT NULL UNIQUE,
    
    CONSTRAINT uq_user_id_website_name UNIQUE (user_id, name)
);
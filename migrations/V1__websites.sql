CREATE OR REPLACE FUNCTION updated_at_now()
RETURNS TRIGGER AS $$
BEGIN
   NEW.updated_at = now(); 
   RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TABLE websites (
    website_id VARCHAR NOT NULL PRIMARY KEY,
    user_id VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    name VARCHAR NOT NULL,
    client_id VARCHAR NOT NULL UNIQUE,
    zitadel_app_id VARCHAR NOT NULL UNIQUE,
    
    CONSTRAINT uq_user_id_website_name UNIQUE (user_id, name)
);

CREATE TRIGGER update_shops_updated_at BEFORE UPDATE
    ON websites FOR EACH ROW EXECUTE PROCEDURE 
    updated_at_now();

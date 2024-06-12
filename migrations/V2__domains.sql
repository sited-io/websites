CREATE TABLE domains (
    domain VARCHAR NOT NULL PRIMARY KEY,
    website_id VARCHAR NOT NULL REFERENCES websites(website_id),
    user_id VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() ON UPDATE NOW(),
    cloudflare_dns_record_id VARCHAR UNIQUE
);

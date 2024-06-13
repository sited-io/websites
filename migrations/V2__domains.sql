CREATE TABLE domains (
    domain_id SERIAL PRIMARY KEY,
    website_id VARCHAR NOT NULL REFERENCES websites(website_id),
    user_id VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() ON UPDATE NOW(),
    domain VARCHAR NOT NULL,
    status VARCHAR NOT NULL,

    UNIQUE INDEX uq_domains_website_id_domain (website_id, domain),
    INDEX (domain, status)
);

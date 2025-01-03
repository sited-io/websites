CREATE TABLE domains (
    domain_id SERIAL PRIMARY KEY,
    website_id VARCHAR NOT NULL REFERENCES websites(website_id),
    user_id VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    domain VARCHAR NOT NULL,
    status VARCHAR NOT NULL
);

CREATE UNIQUE INDEX uq_domains_website_id_domain
    ON domains(website_id, domain);

CREATE INDEX domains_domain_status_idx
    ON domains(domain, status);

CREATE TRIGGER update_shops_updated_at BEFORE UPDATE
    ON domains FOR EACH ROW EXECUTE PROCEDURE 
    updated_at_now();

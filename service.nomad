job "websites" {
  datacenters = ["dc1"]
  type        = "service"

  group "websites-api" {
    count = 2

    network {
      mode = "bridge"

      port "grpc" {}
    }

    service {
      name = "websites-api"
      port = "grpc"

      connect {
        sidecar_service {
          proxy {
            upstreams {
              destination_name = "nats"
              local_bind_port = 4222
            }
            upstreams {
              destination_name = "postgres-sql"
              local_bind_port  = 5432
            }
            upstreams {
              destination_name = "zitadel"
              local_bind_port = 8080
            }
          }
        }
      }

      check {
        type     = "grpc"
        interval = "20s"
        timeout  = "2s"
      }
    }

    task "websites-api" {
      driver = "docker"

      resources {
        cpu        = 400
        memory     = 400
        memory_max = 800
      }

      vault {
        policies = ["service-websites"]
      }

      template {
        destination = "${NOMAD_SECRETS_DIR}/database_root_cert.crt"
        env         = false 
        change_mode = "restart"
        data        = <<EOF
{{- with secret "kv2/data/services" -}}
{{ .Data.data.DATABASE_ROOT_CERT }}
{{- end -}}
EOF
      }

      template {
        destination = "${NOMAD_SECRETS_DIR}/.env"
        env         = true
        change_mode = "restart"
        data        = <<EOF
{{ with nomadVar "nomad/jobs/websites" }}
RUST_LOG='{{ .RUST_LOG }}'
{{ end }}

HOST='0.0.0.0:{{ env "NOMAD_PORT_grpc" }}'

NATS_HOST='{{ env "NOMAD_UPSTREAM_ADDR_nats" }}'
NATS_USER='{{- with nomadVar "nomad/jobs" -}}{{ .NATS_USER }}{{- end -}}'
NATS_PASSWORD='{{- with secret "kv2/data/services" -}}{{ .Data.data.NATS_PASSWORD }}{{- end -}}'

DB_HOST='{{ env "NOMAD_UPSTREAM_IP_postgres-sql" }}'
DB_PORT='{{ env "NOMAD_UPSTREAM_PORT_postgres-sql" }}'
DB_DBNAME='websites'
DB_USER='websites_user'
DB_PASSWORD='{{- with secret "database/static-creds/websites_user" -}}{{ .Data.password }}{{- end -}}'

{{ with nomadVar "nomad/jobs/" }}
JWKS_HOST='{{ .JWKS_HOST_V2 }}'
{{ end }}
JWKS_URL='http://{{ env "NOMAD_UPSTREAM_ADDR_zitadel" }}/oauth/v2/keys'

{{ with nomadVar "nomad/jobs/websites" }}
MAIN_DOMAIN='{{ .MAIN_DOMAIN }}'
FALLBACK_DOMAIN='{{ .FALLBACK_DOMAIN }}'
{{ end }}

ZITADEL_API_URL='http://{{ env "NOMAD_UPSTREAM_ADDR_zitadel" }}/'
{{ with nomadVar "nomad/jobs/websites" }}
ZITADEL_PROJECT_ID='{{ .ZITADEL_PROJECT_ID }}'
{{ end }}
{{ with secret "kv2/data/services/websites" }}
ZITADEL_API_TOKEN='{{ .Data.data.ZITADEL_API_TOKEN }}'
{{ end }}

{{ with nomadVar "nomad/jobs/websites" }}
CLOUDFLARE_API_URL='{{ .CLOUDFLARE_API_URL }}'
CLOUDFLARE_ZONE_ID='{{ .CLOUDFLARE_ZONE_ID }}'
{{ end }}
{{ with secret "kv2/data/services/websites" }}
CLOUDFLARE_API_TOKEN='{{ .Data.data.CLOUDFLARE_API_TOKEN }}'
{{ end }}

{{ with nomadVar "nomad/jobs/websites" }}
BUCKET_NAME='{{ .BUCKET_NAME }}'
BUCKET_ENDPOINT='{{ .BUCKET_ENDPOINT }}'
BUCKET_URL='{{ .BUCKET_URL }}'
IMAGE_MAX_SIZE='{{ .IMAGE_MAX_SIZE }}'
{{ end }}
{{ with secret "kv2/data/services/websites" }}
BUCKET_ACCESS_KEY_ID='{{ .Data.data.BUCKET_ACCESS_KEY_ID }}'
BUCKET_SECRET_ACCESS_KEY='{{ .Data.data.BUCKET_SECRET_ACCESS_KEY }}'
{{ end }}
EOF
      }

      config {
        image      = "__IMAGE__"
        force_pull = true
      }
    }
  }
}

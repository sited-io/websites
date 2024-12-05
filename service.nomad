job "websites" {
  datacenters = ["dc1"]
  type        = "service"

  group "websites-api" {
    count = 1

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
              destination_name = "postgres-sql"
              local_bind_port  = 5432
            }
          }
        }
      }
    }

    task "websites-api" {
      driver = "docker"

      resources {
        cpu        = 100
        memory     = 256
        memory_max = 256
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

{{ with nomadVar "nomad/jobs/websites"}}
DB_HOST='{{ .DB_HOST }}'
DB_PORT='{{ .DB_PORT }}'
DB_DBNAME='{{ .DB_DBNAME }}'
DB_USER='{{ .DB_USER }}'
{{ end }}
DB_ROOT_CERT='{{ env "NOMAD_SECRETS_DIR" }}/database_root_cert.crt'
{{ with secret "kv2/data/services/websites" }}
DB_PASSWORD='{{ .Data.data.DB_PASSWORD }}'
{{ end }}

{{ with nomadVar "nomad/jobs/" }}
JWKS_HOST='{{ .JWKS_HOST }}'
JWKS_URL='{{ .JWKS_URL }}'
{{ end }}

{{ with nomadVar "nomad/jobs/websites" }}
MAIN_DOMAIN='{{ .MAIN_DOMAIN }}'
FALLBACK_DOMAIN='{{ .FALLBACK_DOMAIN }}'
{{ end }}

{{ with nomadVar "nomad/jobs/websites" }}
ZITADEL_API_URL='{{ .ZITADEL_API_URL }}'
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

{{ with nomadVar "nomad/jobs/websites" }}
NATS_HOST='{{ .NATS_HOST }}'
NATS_USER='{{ .NATS_USER }}'
{{ end }}
{{ with secret "kv2/data/services/websites" }}
NATS_PASSWORD='{{ .Data.data.NATS_PASSWORD }}'
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

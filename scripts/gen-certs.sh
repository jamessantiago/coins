#!/usr/bin/env bash
set -euo pipefail

CERT_DIR="certs/localhost"
CERT_FILE="$CERT_DIR/cert.pem"
KEY_FILE="$CERT_DIR/key.pem"
ENV_FILE=".env"

mkdir -p "$CERT_DIR"

openssl req -x509 \
    -newkey rsa:4096 \
    -keyout "$KEY_FILE" \
    -out "$CERT_FILE" \
    -days 365 \
    -nodes \
    -subj "/CN=localhost" 2>/dev/null

chmod 600 "$KEY_FILE"

# Ensure env file ends with newline
[ -f "$ENV_FILE" ] && [ -s "$ENV_FILE" ] && tail -c1 "$ENV_FILE" | read -r _ || echo >> "$ENV_FILE"

# Add/update env vars
for var in "tls_cert_path=$CERT_FILE" "tls_key_path=$KEY_FILE"; do
    key="${var%%=*}"
    if grep -q "^${key}=" "$ENV_FILE" 2>/dev/null; then
        sed -i "s|^${key}=.*|${var}|" "$ENV_FILE"
    else
        echo "$var" >> "$ENV_FILE"
    fi
done

echo "Generated $CERT_DIR certificates and updated $ENV_FILE"

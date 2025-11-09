#!/bin/bash
set -e

echo "[fks_meta] Starting service..."

# Wait for dependencies if needed
# (e.g., wait for MT5 terminal to be available)

# Run the service
exec /app/fks_meta


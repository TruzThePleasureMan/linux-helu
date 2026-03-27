#!/bin/bash
export DATABASE_URL=postgres://helu:password@localhost/helu
sudo su postgres -c "psql -c \"CREATE USER helu WITH PASSWORD 'password';\""
sudo su postgres -c "psql -c \"CREATE DATABASE helu OWNER helu;\""
cargo install sqlx-cli --no-default-features --features postgres
cd helu-server && cargo sqlx prepare

#!/bin/bash
sudo apt-get update && sudo apt-get install -y postgresql
sudo systemctl start postgresql
sudo -u postgres psql -c "CREATE USER helu WITH PASSWORD 'password';"
sudo -u postgres psql -c "CREATE DATABASE helu OWNER helu;"

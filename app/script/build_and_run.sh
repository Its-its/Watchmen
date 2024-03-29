#!/usr/bin/env bash

cd $(dirname "$0")

cd ../..

echo "Retrieving latest GIT"

git pull

echo "Compiling Angular"

cd frontend
npm install
node_modules/.bin/ng build --configuration development
cd ..

echo "Running Watchmen"

cd backend
cargo run --release
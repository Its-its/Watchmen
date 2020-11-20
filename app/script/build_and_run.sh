cd ../..

echo "Retrieving latest GIT"

git pull

echo "Compiling SASS files"

sass ./app/scss/core.scss ./app/compiled/css/core.css

echo "Compiling Typescript files"

npx tsc

echo "Running Watchmen"

cargo run
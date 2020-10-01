@echo off

start wt --title Backend -d ../.. cmd.exe /k cargo watch -x run --watch src ;^
 split-pane --title Frontend -d ../.. cmd.exe /k tsc -w -p tsconfig.json ;^
 split-pane -H --title SCSS -d .. cmd.exe /k sass --watch ./scss/core.scss ./compiled/css/core.css ;^
 focus-tab -t 0
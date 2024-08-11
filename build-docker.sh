#!/bin/bash
docker rm -f revkitty-test-victim-server
docker image rm revkitty-test-victim-server
docker build -t revkitty-test-victim-server .
# docker run --name=revkitty-test-victim-server --rm -i revkitty-test-victim-server '/revr-server --ip 172.17.0.1 --port 4444' 
docker run -d revkitty-test-victim-server
# --rm -i revkitty-test-victim-server '/revr-server --ip 172.17.0.1 --port 4444' 

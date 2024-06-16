#!/bin/bash
docker rm -f revkitty-test-victim-server
docker image rm revkitty-test-victim-server
docker build -t revkitty-test-victim-server .
docker run --name=revkitty-test-victim-server --rm -it revkitty-test-victim-server /bin/sh 

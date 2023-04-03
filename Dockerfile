FROM ubuntu:22.04

COPY target/debug/bsc_board /bin/bsc_board 

EXPOSE 8080

CMD ["/bin/bsc_board"]
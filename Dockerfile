# We use debian 9 to allow our binary distribution to run on a lower version of libc.
# This increases our compatiblity.
FROM debian:9

WORKDIR /src/gcsthin

# Few essential things before we can get going
RUN apt-get update
RUN apt-get install -y build-essential pkg-config sudo curl git python3 libssl-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV CARGO=/root/.cargo/bin/cargo

COPY . .
RUN make test
RUN make

RUN tar --transform "s|^|gcsthin/|" -cJf gcsthin.tar.xz gcsthin

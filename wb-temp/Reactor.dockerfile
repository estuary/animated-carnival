FROM ubuntu:jammy


# docker build -f Reactor.dockerfile -t reactor:whb .

RUN apt update -y \
    && apt install --no-install-recommends -y \
    ca-certificates \
    curl

RUN curl -o docker-cli.deb 'https://download.docker.com/linux/ubuntu/dists/jammy/pool/stable/amd64/docker-ce-cli_20.10.17~3-0~ubuntu-jammy_amd64.deb' && \
    dpkg -i docker-cli.deb && \
    rm docker-cli.deb


# A Fedora 24 BDCS API Container
FROM fedora:24
MAINTAINER Brian C. Lane <bcl@redhat.com>

RUN dnf install -y dnf-plugins-core gnupg tar git sudo curl file gcc-c++ gcc gdb glibc-devel openssl-devel make xz sqlite-devel openssl-devel

RUN curl -sSf https://static.rust-lang.org/rustup.sh \
  | sh -s -- --yes --disable-sudo --channel=nightly \
  && rustc --version && cargo --version

ENV CARGO_HOME /cargo
ENV SRC_PATH /src

RUN mkdir -p "$CARGO_HOME" "$SRC_PATH"
WORKDIR $SRC_PATH

RUN echo 'PATH=/usr/local/bin/:$PATH' >> /etc/bashrc

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
EXPOSE 4000

# Volumes for database and recipe storage.
VOLUME /mddb /bdcs-recipes

## Do the things more likely to change below here. ##

COPY . /bdcs-api-rs/
RUN cd /bdcs-api-rs/ && cargo build

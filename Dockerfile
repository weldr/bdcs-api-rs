# A Fedora 24 BDCS API Container
FROM fedora:24
MAINTAINER Brian C. Lane <bcl@redhat.com>

RUN dnf install -y dnf-plugins-core gnupg tar git sudo curl file gcc-c++ gcc gdb glibc-devel openssl-devel make xz sqlite-devel openssl-devel

RUN curl https://sh.rustup.rs -sSf \
  | sh -s -- -y --default-toolchain nightly
ENV PATH="/root/.cargo/bin:${PATH}"

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
EXPOSE 4000

# Volumes for database and recipe storage.
VOLUME /mddb /bdcs-recipes

## Do the things more likely to change below here. ##
## Run rustup update to pick up the latest nightly ##
COPY . /bdcs-api-rs/
RUN cd /bdcs-api-rs/ && rustup update && cargo build

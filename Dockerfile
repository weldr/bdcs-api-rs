# A Fedora 25 BDCS API Container
FROM welder/fedora:latest
MAINTAINER Brian C. Lane <bcl@redhat.com>

# NOTE: if you need updated rustc then make sure to update this line
RUN curl https://sh.rustup.rs -sSf \
  | sh -s -- -y --default-toolchain nightly-2017-08-01
ENV PATH="/root/.cargo/bin:${PATH}"

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
EXPOSE 4000

# Volumes for database and recipe storage.
VOLUME /mddb /bdcs-recipes /mockfiles

RUN dnf --setopt=deltarpm=0 --verbose -y install python-toml && dnf clean all

## Do the things more likely to change below here. ##
RUN mkdir /bdcs-api-rs/
COPY parse-cargo-toml.py /bdcs-api-rs/

# Manually install cargo dependencies before building
# so we can have a reusable intermediate container.
# This workaround is needed until cargo can do this by itself:
# https://github.com/rust-lang/cargo/issues/2644
# https://github.com/rust-lang/cargo/pull/3567
COPY Cargo.toml /bdcs-api-rs/
WORKDIR /bdcs-api-rs/
RUN python ./parse-cargo-toml.py | while read cmd; do \
        $cmd;                                    \
    done

# NOTE: WORKDIR is already /bdcs-api-rs/
COPY . .
RUN make bdcs-api

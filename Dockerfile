# A Fedora 25 BDCS API Container
FROM welder/fedora:latest
MAINTAINER Brian C. Lane <bcl@redhat.com>

# NOTE: if you need updated rustc then make sure to update this line
RUN curl https://sh.rustup.rs -sSf \
  | sh -s -- -y --default-toolchain nightly-2017-09-06
ENV PATH="/root/.cargo/bin:${PATH}"

EXPOSE 4000

# Volumes for database and recipe storage.
VOLUME /mddb /bdcs-recipes /mockfiles

# testing dependencies which don't belong here but this
# is the best place to avoid executing these steps on every single build
# when intermediate container cache is available
RUN cargo install clippy --vers 0.0.151
RUN dnf --setopt=deltarpm=0 --verbose -y install \
    pylint python-toml python-nose-parameterized \
    elfutils-devel binutils-devel &&             \
    dnf clean all

ENV PATH="~/.local/bin:${PATH}"
RUN curl https://codeload.github.com/SimonKagstrom/kcov/tar.gz/master > master.tar.gz
RUN tar -xzf master.tar.gz
RUN mkdir kcov-master/build && cd kcov-master/build && cmake -DCMAKE_INSTALL_PREFIX:PATH=~/.local .. && make && make install
RUN rm -rf *.tar.gz kcov-master/

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

# NOTE: this must be at the bottom otherwise it invalidates
# the intermediate Docker layers after it
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

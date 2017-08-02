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

## Do the things more likely to change below here. ##
COPY . /bdcs-api-rs/
RUN make -C /bdcs-api-rs/ bdcs-api doc

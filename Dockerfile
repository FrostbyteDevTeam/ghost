# Ghost MCP server, headless.
#
# This image exists so registries and CI can start ghost-mcp and introspect it
# (initialize, tools/list) without a desktop: the server answers both with no
# DISPLAY and no accessibility bus. It is not how people drive a desktop with
# Ghost - a container has no windows to control. For real use install the
# release archive or the .mcpb bundle (see README).
#
# The image does not compile the workspace. It downloads the Linux release
# whose version matches [workspace.package] in Cargo.toml and verifies the
# published sha256, so the binary is the exact one users get. Override with
# --build-arg GHOST_VERSION=x.y.z to pin a different release.

FROM ubuntu:24.04 AS fetch
ARG GHOST_VERSION=
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /fetch
COPY Cargo.toml ./
RUN set -eu; \
    v="${GHOST_VERSION}"; \
    if [ -z "$v" ]; then \
        v="$(sed -n '/^\[workspace\.package\]/,/^\[/{s/^version *= *"\([^"]*\)".*/\1/p}' Cargo.toml | head -n1)"; \
    fi; \
    test -n "$v"; \
    echo "ghost ${v}"; \
    base="https://github.com/NORTHTEKDevs/ghost/releases/download/v${v}"; \
    curl -fsSL -o ghost.tar.gz "${base}/ghost-linux-x86_64.tar.gz"; \
    curl -fsSL -o ghost.tar.gz.sha256 "${base}/ghost-linux-x86_64.tar.gz.sha256"; \
    echo "$(cut -d' ' -f1 ghost.tar.gz.sha256)  ghost.tar.gz" | sha256sum -c -; \
    mkdir unpacked; \
    tar xzf ghost.tar.gz -C unpacked; \
    mv "$(find unpacked -type f -name ghost-mcp | head -n1)" ghost-mcp; \
    chmod 0755 ghost-mcp

# glibc 2.39 is the floor for the release binary; ubuntu:24.04 ships exactly that.
FROM ubuntu:24.04
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 ghost
COPY --from=fetch /fetch/ghost-mcp /usr/local/bin/ghost-mcp
USER ghost
ENV GHOST_SHELL=off
ENTRYPOINT ["/usr/local/bin/ghost-mcp"]

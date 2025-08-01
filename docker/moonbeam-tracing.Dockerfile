# Node for tracing
#
# Requires to run from repository root and to copy the binary in the build folder (part of the CI workflow)

FROM docker.io/library/ubuntu:22.04 AS builder

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

FROM debian:bookworm-slim
LABEL maintainer="alan@moonsonglabs.com"
LABEL description="Binary for Moonbeam Tracing Nodes"

RUN useradd -m -u 1000 -U -s /bin/sh -d /moonbeam moonbeam && \
	mkdir -p /moonbeam/.local/share && \
	mkdir /data && \
	chown -R moonbeam:moonbeam /data && \
	ln -s /data /moonbeam/.local/share/moonbeam && \
	rm -rf /usr/sbin

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

USER moonbeam

COPY --chown=moonbeam build/moonbeam /moonbeam/moonbeam
COPY --chown=moonbeam build/moonbase-substitutes-tracing /moonbeam/moonbase-substitutes-tracing
COPY --chown=moonbeam build/moonriver-substitutes-tracing /moonbeam/moonriver-substitutes-tracing
COPY --chown=moonbeam build/moonbeam-substitutes-tracing /moonbeam/moonbeam-substitutes-tracing
RUN chmod uog+x /moonbeam/moonbeam

# 30333 for parachain p2p 
# 30334 for relaychain p2p 
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 30334 9933 9944 9615 

VOLUME ["/data"]

ENTRYPOINT ["/moonbeam/moonbeam"]

FROM gitpod/workspace-full

USER gitpod

ENV DEBIAN_FRONTEND=noninteractive

RUN cargo install cargo-reaper --git https://github.com/MomoLangenstein/grim-reaper -f

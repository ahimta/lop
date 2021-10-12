FROM rust:1.55

RUN rustup component add rustfmt

CMD /project/scripts/continuous-integration.sh

FROM alpine:latest AS builder
ARG TARGETARCH
WORKDIR /builder
COPY . .
RUN ./scripts/binary.sh $TARGETARCH && \
    echo "nobody:x:65534:65534:Nobody:/:" > /etc_passwd

FROM scratch
COPY --from=builder --chmod=755 /builder/listening-to ./listening-to
COPY --from=builder "/etc_passwd" "/etc/passwd"
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /usr/local/ssl/ca-certificates.crt
USER nobody

ENV RUST_LOG=info
ENV SPOTIFY_CLIENT_ID=your-client-id
ENV SPOTIFY_CLIENT_SECRET=your-client-secret

ENV SLACK_BASE_URL=https://slack.com
ENV SLACK_TOKEN=your-slack-token
ENV SLACK_COOKIE=your-slack-cookie
ENV CRON_SCHEDULE="*/10 * 8-18 * * 1-5"

CMD ["./listening-to"]
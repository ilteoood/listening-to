FROM alpine:latest AS builder
ARG TARGETARCH
WORKDIR /builder
COPY . .
RUN ./scripts/binary.sh $TARGETARCH

FROM scratch
COPY --from=builder --chmod=755 /builder/listening-to listening-to

ENV RUST_LOG=info
ENV SPOTIFY_CLIENT_ID=your-client-id
ENV SPOTIFY_CLIENT_SECRET=your-client-secret

ENV SLACK_BASE_URL=https://slack.com
ENV SLACK_TOKEN=your-slack-token
ENV SLACK_COOKIE=your-slack-cookie
ENV CRON_SCHEDULE="*/10 * 8-18 * * 1-5"

CMD ["listening-to"]
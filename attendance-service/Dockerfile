FROM golang:1.22-alpine AS builder

RUN apk add --no-cache git ca-certificates

WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download

COPY . .

RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 \
    go build -ldflags="-w -s" \
    -o /bin/attendance-service \
    ./cmd/attendance-service

FROM gcr.io/distroless/static-debian12:nonroot

COPY --from=builder /bin/attendance-service /attendance-service
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

USER nonroot:nonroot

EXPOSE 8080
EXPOSE 9090

ENTRYPOINT ["/attendance-service"]

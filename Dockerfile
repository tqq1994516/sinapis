FROM alpine as base

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.aliyun.com/g' /etc/apk/repositories
RUN apk update
RUN apk add protobuf grpc

ARG PERSON_CENTER=person-center
ARG FRONTEND_BASE_SERVICE=frontend-base-service
ARG AGGREGATION_GATEWAY=aggregation-gateway

WORKDIR /app

FROM base as person-center

COPY target/release/${PERSON_CENTER} .

EXPOSE 50000

CMD ["./${PERSON_CENTER}"]

FROM base as frontend-base-service

COPY target/release/${FRONTEND_BASE_SERVICE} .

EXPOSE 50000

CMD ["./${FRONTEND_BASE_SERVICE}"]

FROM base as aggregation-gateway

COPY target/release/${AGGREGATION_GATEWAY} .

EXPOSE 80

CMD ["./${AGGREGATION_GATEWAY}"]

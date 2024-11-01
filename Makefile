.PHONY: clean all all_image all_push

RELEASE ?= false
HARBOR_HOST = 192.168.33.48
APP = semen-sinapis
PERSON_CENTER = person-center
FRONTEND_BASE_SERVICE = frontend-base-service
AGGREGATION_GATEWAY = aggregation-gateway
PARAMETER_ANALYSIS = parameter-analysis

all: person_center frontend_base_service aggregation_gateway

all_image: person_center_image frontend_base_service_image aggregation_gateway_image

all_push: person_center_push frontend_base_service_push aggregation_gateway_push

clean:
	cargo clean

ifeq ($(RELEASE),true)
person_center:
	cd ${PERSON_CENTER} && cargo build --target wasm32-wasi --release
else
person_center:
	cd ${PERSON_CENTER} && cargo build --target wasm32-wasi
endif

ifeq ($(RELEASE),true)
person_center_image: person_center
	podman build --target ${PERSON_CENTER} -t ${PERSON_CENTER}:${VERSION} .
else
person_center_image: person_center
	podman build -f Dockerfile.debug --target ${PERSON_CENTER} -t ${PERSON_CENTER}:${VERSION} .
endif

person_center_push: person_center_image
	podman tag ${PERSON_CENTER}:${VERSION} ${HARBOR_HOST}/library/${APP}/${PERSON_CENTER}:${VERSION}
	podman push ${HARBOR_HOST}/library/${APP}/${PERSON_CENTER}:${VERSION}

ifeq ($(RELEASE),true)
frontend_base_service:
	cd ${FRONTEND_BASE_SERVICE} && cargo build --target wasm32-wasi --release
else
frontend_base_service:
	cd ${FRONTEND_BASE_SERVICE} && cargo build --target wasm32-wasi
endif

ifeq ($(RELEASE),true)
frontend_base_service_image: frontend_base_service
	podman build --target ${FRONTEND_BASE_SERVICE} -t ${FRONTEND_BASE_SERVICE}:${VERSION} .
else
frontend_base_service_image: frontend_base_service
	podman build -f Dockerfile.debug --target ${FRONTEND_BASE_SERVICE} -t ${FRONTEND_BASE_SERVICE}:${VERSION} .
endif

frontend_base_service_push: frontend_base_service_image
	podman tag ${FRONTEND_BASE_SERVICE}:${VERSION} ${HARBOR_HOST}/library/${APP}/${FRONTEND_BASE_SERVICE}:${VERSION}
	podman push ${HARBOR_HOST}/library/${APP}/${FRONTEND_BASE_SERVICE}:${VERSION}

ifeq ($(RELEASE),true)
aggregation_gateway:
	cd ${AGGREGATION_GATEWAY} && cargo build --target wasm32-wasi --release
else
aggregation_gateway:
	cd ${AGGREGATION_GATEWAY} && cargo build --target wasm32-wasi
endif

ifeq ($(RELEASE),true)
aggregation_gateway_image: aggregation_gateway
	podman build --target ${AGGREGATION_GATEWAY} -t ${AGGREGATION_GATEWAY}:${VERSION} .
else
aggregation_gateway_image: aggregation_gateway
	podman build -f Dockerfile.debug --target ${AGGREGATION_GATEWAY} -t ${AGGREGATION_GATEWAY}:${VERSION} .
endif

aggregation_gateway_push: aggregation_gateway_image
	podman tag ${AGGREGATION_GATEWAY}:${VERSION} ${HARBOR_HOST}/library/${APP}/${AGGREGATION_GATEWAY}:${VERSION}
	podman push ${HARBOR_HOST}/library/${APP}/${AGGREGATION_GATEWAY}:${VERSION}

parameter_analysis_image: 
	podman build -f Dockerfile.actor --target ${PARAMETER_ANALYSIS} -t ${PARAMETER_ANALYSIS}:${VERSION} .

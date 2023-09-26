.PHONY: clean all all_image all_apply

RELEASE ?= false
HARBOR_HOST = 192.168.33.48
APP = semen-sinapis
PERSON_CENTER = person-center
FRONTEND_BASE_SERVICE = frontend-base-service
AGGREGATION_GATEWAY = aggregation-gateway
PARAMETER_ANALYSIS = parameter-analysis

all: person_center frontend_base_service aggregation_gateway

all_image: person_center_image frontend_base_service_image aggregation_gateway_image

all_apply: person_center_apply frontend_base_service_apply aggregation_gateway_apply

clean:
	cargo clean

ifeq ($(RELEASE),true)
person_center:
	cd ${PERSON_CENTER} && cargo build --release
else
person_center:
	cd ${PERSON_CENTER} && cargo build
endif

ifeq ($(RELEASE),true)
person_center_image: person_center
	docker build --target ${PERSON_CENTER} -t ${PERSON_CENTER}:${VERSION} .
else
person_center_image: person_center
	docker build -f Dockerfile.debug --target ${PERSON_CENTER} -t ${PERSON_CENTER}:${VERSION} .
endif

person_center_apply: person_center_image
	docker tag ${PERSON_CENTER}:${VERSION} ${HARBOR_HOST}/library/${APP}/${PERSON_CENTER}:${VERSION}
	docker push ${HARBOR_HOST}/library/${APP}/${PERSON_CENTER}:${VERSION}
	cd manifests && kubectl apply -f ${PERSON_CENTER}.yaml

ifeq ($(RELEASE),true)
frontend_base_service:
	cd ${FRONTEND_BASE_SERVICE} && cargo build --release
else
frontend_base_service:
	cd ${FRONTEND_BASE_SERVICE} && cargo build
endif

ifeq ($(RELEASE),true)
frontend_base_service_image: frontend_base_service
	docker build --target ${FRONTEND_BASE_SERVICE} -t ${FRONTEND_BASE_SERVICE}:${VERSION} .
else
frontend_base_service_image: frontend_base_service
	docker build -f Dockerfile.debug --target ${FRONTEND_BASE_SERVICE} -t ${FRONTEND_BASE_SERVICE}:${VERSION} .
endif

frontend_base_service_apply: frontend_base_service_image
	docker tag ${FRONTEND_BASE_SERVICE}:${VERSION} ${HARBOR_HOST}/library/${APP}/${FRONTEND_BASE_SERVICE}:${VERSION}
	docker push ${HARBOR_HOST}/library/${APP}/${FRONTEND_BASE_SERVICE}:${VERSION}
	cd manifests && kubectl apply -f ${FRONTEND_BASE_SERVICE}.yaml

ifeq ($(RELEASE),true)
aggregation_gateway:
	cd ${AGGREGATION_GATEWAY} && cargo build --release
else
aggregation_gateway:
	cd ${AGGREGATION_GATEWAY} && cargo build
endif

ifeq ($(RELEASE),true)
aggregation_gateway_image: aggregation_gateway
	docker build --target ${AGGREGATION_GATEWAY} -t ${AGGREGATION_GATEWAY}:${VERSION} .
else
aggregation_gateway_image: aggregation_gateway
	docker build -f Dockerfile.debug --target ${AGGREGATION_GATEWAY} -t ${AGGREGATION_GATEWAY}:${VERSION} .
endif

aggregation_gateway_apply: frontend_base_service_image
	docker tag ${AGGREGATION_GATEWAY}:${VERSION} ${HARBOR_HOST}/library/${APP}/${AGGREGATION_GATEWAY}:${VERSION}
	docker push ${HARBOR_HOST}/library/${APP}/${AGGREGATION_GATEWAY}:${VERSION}
	cd manifests && kubectl apply -f ${AGGREGATION_GATEWAY}.yaml

parameter_analysis_image: 
	docker build -f Dockerfile.actor --target ${PARAMETER_ANALYSIS} -t ${PARAMETER_ANALYSIS}:${VERSION} .

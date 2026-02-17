# Variables
IMAGE_NAME := relay
VERSION := latest
DOCKER_USER := $(shell whoami)
PORT := 4000
DOCS_PORT := 3000
DOCS_BUCKET := relay-http-website

.PHONY: help build run run-custom stop clean push tag docs docs-deploy

help:
	@echo "Available targets:"
	@echo "  build        - Build the Docker image"
	@echo "  run          - Run container with default config"
	@echo "  run-custom   - Run container with custom config.toml"
	@echo "  stop         - Stop running container"
	@echo "  clean        - Remove Docker image"
	@echo "  tag          - Tag image for Docker Hub (use DOCKER_USER=username)"
	@echo "  push         - Push image to Docker Hub (use DOCKER_USER=username)"
	@echo "  docs         - Serve documentation locally"
	@echo "  docs-deploy  - Deploy docs to S3"
	@echo ""
	@echo "Examples:"
	@echo "  make build"
	@echo "  make run PORT=8080"
	@echo "  make tag DOCKER_USER=myusername VERSION=v1.0.0"
	@echo "  make push DOCKER_USER=myusername"
	@echo "  make docs"
	@echo "  make docs-deploy DOCS_BUCKET=my-relay-docs"

build:
	docker build -t $(IMAGE_NAME):$(VERSION) .

run:
	docker run -d --name $(IMAGE_NAME) -p $(PORT):$(PORT) $(IMAGE_NAME):$(VERSION)
	@echo "Container started on port $(PORT)"

run-custom:
	docker run -d --name $(IMAGE_NAME) -p $(PORT):$(PORT) \
	-v $(PWD)/config.toml:/app/config.toml \
	$(IMAGE_NAME):$(VERSION)
	@echo "Container started on port $(PORT) with custom config"

stop:
	docker stop $(IMAGE_NAME) || true
	docker rm $(IMAGE_NAME) || true

clean: stop
	docker rmi $(IMAGE_NAME):$(VERSION) || true

tag:
	docker tag $(IMAGE_NAME):$(VERSION) $(DOCKER_USER)/$(IMAGE_NAME):$(VERSION)

push: tag
	docker push $(DOCKER_USER)/$(IMAGE_NAME):$(VERSION)

docs:
	@echo "Starting documentation server at http://localhost:$(DOCS_PORT)"
	@echo "Press Ctrl+C to stop"
	@cd docs && python3 -m http.server $(DOCS_PORT)

docs-deploy:
	@echo "Deploying docs to S3 bucket: $(DOCS_BUCKET)"
	aws s3 sync docs/ s3://$(DOCS_BUCKET) --delete --cache-control "max-age=3600"
	@echo "Deployment complete!"

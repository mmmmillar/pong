IMAGE_NAME := "pong"
CONTAINER_NAME := "pong"

# Build the Docker image
build:
	docker build --no-cache -t {{IMAGE_NAME}} .

# Run the Docker container
run:
	docker run -d -p 3030:3030 --name {{CONTAINER_NAME}} {{IMAGE_NAME}}

# Stop and remove the Docker container
stop:
	docker stop {{CONTAINER_NAME}} || true
	docker rm {{CONTAINER_NAME}} || true

# Clean up Docker artifacts
clean: stop
	docker rmi {{IMAGE_NAME}} || true

# Rebuild and run the Docker image
build-and-run: stop build run

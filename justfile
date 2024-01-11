set dotenv-load

AWS_ACCOUNT_ID := "$AWS_ACCOUNT_ID"
ECR_REPO_BASE := "$AWS_ACCOUNT_ID.dkr.ecr.eu-west-2.amazonaws.com"
IMAGE_NAME := "$AWS_ACCOUNT_ID.dkr.ecr.eu-west-2.amazonaws.com/pong:latest"
CONTAINER_NAME := "pong"

# Build the Docker image
build:
	docker build -t {{IMAGE_NAME}} .

# Run the Docker container
run:
	docker run -d -p 80:3030 --name {{CONTAINER_NAME}} {{IMAGE_NAME}}

# Stop and remove the Docker container
stop:
	docker stop {{CONTAINER_NAME}} || true
	docker rm {{CONTAINER_NAME}} || true

# Clean up Docker artifacts
clean:
	stop
	docker rmi {{IMAGE_NAME}} || true

# Rebuild and run the Docker image
build-and-run:
	stop
	build
	run

# Terraform plan
plan:
	terraform plan

# Deploy infra, push image and run
deploy:
	terraform apply -auto-approve
	aws ecr get-login-password --region eu-west-2 | docker login --username AWS --password-stdin {{ECR_REPO_BASE}}
	docker push {{IMAGE_NAME}}
	echo "aws ecr get-login-password --region eu-west-2 | docker login --username AWS --password-stdin {{ECR_REPO_BASE}} && docker pull {{IMAGE_NAME}} && docker stop {{CONTAINER_NAME}} || true && docker rm {{CONTAINER_NAME}} || true && docker run -d -p 80:3030 --name {{CONTAINER_NAME}} {{IMAGE_NAME}}" | ssh -i "matt.pem" admin@$(terraform output -raw ec2_public_dns)
	echo "http://$(terraform output -raw ec2_public_dns)"

# Restart remote
restart:
		echo "aws ecr get-login-password --region eu-west-2 | docker stop {{CONTAINER_NAME}} || true && docker rm {{CONTAINER_NAME}} || true && docker run -d -p 80:3030 --name {{CONTAINER_NAME}} {{IMAGE_NAME}}" | ssh -i "matt.pem" admin@$(terraform output -raw ec2_public_dns)

# Login to remote
ssh:
	ssh -i "matt.pem" admin@$(terraform output -raw ec2_public_dns)

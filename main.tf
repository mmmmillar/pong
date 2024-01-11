provider "aws" {
  region = "eu-west-2"
  profile= "default"
}

terraform {
  backend "s3" {
    bucket         = "tf-buckett"
    key            = "pong/terraform.tfstate"
    region         = "eu-west-2"
  }
}

# ECR Repository
resource "aws_ecr_repository" "pong_repository" {
  name      = "pong"
  image_tag_mutability = "MUTABLE"
  force_delete = true
  image_scanning_configuration {
    scan_on_push = true
  }
}

# ECR IAM
resource "aws_iam_policy" "ecr_policy" {
  name        = "ECRReadOnlyAccess"
  description = "IAM policy for read-only access to pong ECR"

  policy = jsonencode({
   "Version":"2012-10-17",
   "Statement": [
      {
        "Sid":"ListImagesInRepository",
        "Effect":"Allow",
        "Action": [
          "ecr:ListImages"
        ],
        "Resource": "${aws_ecr_repository.pong_repository.arn}"
      },
      {
        "Sid":"GetAuthorizationToken",
        "Effect":"Allow",
        "Action": [
          "ecr:GetAuthorizationToken"
        ],
        "Resource":"*"
      },
      {
        "Sid":"ManageRepositoryContents",
        "Effect":"Allow",
        "Action": [
          "ecr:BatchCheckLayerAvailability",
          "ecr:GetDownloadUrlForLayer",
          "ecr:GetRepositoryPolicy",
          "ecr:DescribeRepositories",
          "ecr:ListImages",
          "ecr:DescribeImages",
          "ecr:BatchGetImage",
          "ecr:InitiateLayerUpload",
          "ecr:UploadLayerPart",
          "ecr:CompleteLayerUpload",
          "ecr:PutImage"
        ],
        "Resource": "${aws_ecr_repository.pong_repository.arn}"
      }
    ]
  })
}

resource "aws_iam_role" "ecr_role" {
  name = "ECRReadOnlyRole"

  assume_role_policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        Action = "sts:AssumeRole",
        Effect = "Allow",
        Principal = {
          Service = "ec2.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "ecr_role_policy_attachment" {
  policy_arn = aws_iam_policy.ecr_policy.arn
  role       = aws_iam_role.ecr_role.name
}

resource "aws_iam_instance_profile" "ecr_instance_profile" {
  name = "ECRInstanceProfile"
  role = aws_iam_role.ecr_role.name
}
# Security Group
resource "aws_security_group" "pong_security_group" {
  name        = "pong-security-group"

  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Add additional ingress rules as needed

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# EC2 Instance
resource "aws_instance" "pong_instance" {
  ami           = "ami-06bd7f67e90613d1a"
  instance_type = "t2.micro"

  key_name      = "matt"
  vpc_security_group_ids = [aws_security_group.pong_security_group.id]
  iam_instance_profile = aws_iam_instance_profile.ecr_instance_profile.name

  user_data = <<-EOF
  #!/bin/bash
  # Add Docker's official GPG key:
  sudo apt-get update -y
  sudo apt-get install ca-certificates curl gnupg -y
  sudo install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/debian/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
  sudo chmod a+r /etc/apt/keyrings/docker.gpg

  # Add the repository to Apt sources:
  echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
  sudo apt-get update -y

  sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin -y
  sudo systemctl enable docker
  sudo systemctl start docker
  sudo chmod 666 /var/run/docker.sock
  EOF
}

# API Gateway
resource "aws_api_gateway_rest_api" "pong_server" {
  name        = "pong-server"
}

output "ec2_public_dns" {
  value = aws_instance.pong_instance.public_dns
}
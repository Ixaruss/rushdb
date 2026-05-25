pipeline {
    agent any

    environment {
        DOCKERHUB_USER  = 'ixcarus'
        IMAGE_NAME      = "${DOCKERHUB_USER}/rushdb"
        IMAGE_TAG       = "${env.BUILD_NUMBER}"
    }

    stages {

        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Run Tests') {
            agent {
                docker {
                    image 'rust:latest'
                    args  '-v cargo-cache:/usr/local/cargo/registry' // cache deps
                    reuseNode true
                }
            }
            steps {
                sh 'cargo test --verbose'
            }
        }

        stage('Build Docker Image') {
            steps {
                script {
                    dockerImage = docker.build("${IMAGE_NAME}:${IMAGE_TAG}")
                }
            }
        }

        stage('Push to Docker Hub') {
            steps {
                script {
                    docker.withRegistry('https://index.docker.io/v1/', 'DOCKER_HUB_CREDENTIALS') {
                        dockerImage.push("${IMAGE_TAG}")
                        dockerImage.push('latest')       // also tag as latest
                    }
                }
            }
        }
    }

    post {
        success {
            echo "✅ Built and pushed ${IMAGE_NAME}:${IMAGE_TAG}"
        }
        failure {
            echo "❌ Pipeline failed — check the logs above"
        }
        always {
            sh "docker rmi ${IMAGE_NAME}:${IMAGE_TAG} || true"
        }
    }
}

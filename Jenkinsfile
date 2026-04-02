pipeline {
  agent any

  environment {
    CONTAINER = 'rust:trixie'
    MIRROR_REPO = 'https://github.com/EpitechPGE2-2025/G-NWP-400-LIL-4-1-myteams-1.git'
    MIRROR_CRED = 'jenkins-pat'
  }

  stages {
    stage('Checkout') {
      steps { checkout scm }
    }

    stage('Compilation') {
      steps {
        script {
          docker.image(env.CONTAINER).inside('-i') {
            sh '''bash -c "
set -euo pipefail
cd ${WORKSPACE}
cargo check
"'''
          }
        }
      }
    }

    stage('Mirror to repo') {
      when { expression { currentBuild.currentResult == 'SUCCESS' } }
      steps {
        withCredentials([usernamePassword(credentialsId: env.MIRROR_CRED, usernameVariable: 'GIT_USER', passwordVariable: 'GIT_TOKEN')]) {
          sh '''bash -c "
set -e
git config user.name 'jenkins-bot'
git config user.email 'jenkins@inkurey.fr'
# Remove existing mirror remote if present
git remote remove mirror 2>/dev/null || true
# Add correct mirror remote with token
git remote add mirror https://$GIT_USER:$GIT_TOKEN@${MIRROR_REPO#https://}
git fetch mirror || true
# push to main
git push --force-with-lease mirror HEAD:refs/heads/main
"'''
        }
      }
    }
  }

  post {
    failure { echo 'Build or tests failed — not mirroring.' }
    success { echo 'All done — mirrored to target repo.' }
  }
}

pipeline {
    agent {
        kubernetes {
            yamlFile '.deploy/rust-builder-pod.yaml'
        }
    }
    environment {
        RUSTC_WRAPPER = '/usr/local/bin/sccache'
        CI = 'true'
        GITHUB_TOKEN = credentials('ns212-github-token')
        DISCORD_WEBHOOK_URL = credentials('discord_webhook_url')
    }

    options {
        timestamps()
        ansiColor('xterm')
        buildDiscarder(logRotator(artifactDaysToKeepStr: '7', artifactNumToKeepStr: '5'))
    }

    stages {
        stage('Test') {
            steps {
                container('rust') {
                    sh '''
                        rustc --version
                        SCCACHE_START_SERVER=1 SCCACHE_IDLE_TIMEOUT=0 /usr/local/bin/sccache
                        /usr/local/bin/sccache -s
                        cargo fmt -- --check
                        cargo check --workspace --release
                        cargo test --workspace --release
                        /usr/local/bin/sccache -s
                    '''
                }
            }
        }

        stage('Build standalone') {
            steps {
                container('rust') {
                    sh '''
                        SCCACHE_START_SERVER=1 SCCACHE_IDLE_TIMEOUT=0 /usr/local/bin/sccache
                        /usr/local/bin/sccache -s
                        cargo build --release --bin interbtc-standalone
                        cp target/release/interbtc-standalone target/release/interbtc-standalone
                        /usr/local/bin/sccache -s
                    '''
                    archiveArtifacts 'target/release/interbtc-standalone'
                    stash(name: 'build-standalone', includes: 'Dockerfile_release, target/release/interbtc-standalone')
                }
            }
        }

        stage('Build parachain') {
            steps {
                container('rust') {
                    sh '''
                        SCCACHE_START_SERVER=1 SCCACHE_IDLE_TIMEOUT=0 /usr/local/bin/sccache
                        /usr/local/bin/sccache -s
                        cargo build --release --bin interbtc-parachain
                        /usr/local/bin/sccache -s
                    '''
                    archiveArtifacts 'target/release/interbtc-parachain'
                    stash(name: 'build-parachain', includes: 'Dockerfile_release, target/release/interbtc-parachain')
                }
            }
        }

        stage('Make Image - standalone') {
            when { anyOf { branch 'master'; tag '*' } }
            environment {
                PATH = "/busybox:$PATH"
                REGISTRY = 'registry.gitlab.com'
                REPOSITORY = 'interlay/interbtc'
                IMAGE = 'standalone'
            }
            steps {
                container(name: 'kaniko', shell: '/busybox/sh') {
                    dir('unstash') {
                        unstash('build-standalone')
                        runKaniko()
                    }
                }
            }
        }

        stage('Make Image - parachain') {
            when { anyOf { branch 'master'; tag '*' } }
            environment {
                PATH = "/busybox:$PATH"
                REGISTRY = 'registry.gitlab.com'
                REPOSITORY = 'interlay/interbtc'
                IMAGE = 'parachain'
            }
            steps {
                container(name: 'kaniko', shell: '/busybox/sh') {
                    dir('unstash') {
                        unstash('build-parachain')
                        runKaniko()
                    }
                }
            }
        }

        stage('Create GitHub release') {
            when { tag '*' }
            steps {
                sh '''
                    wget -q -O - https://github.com/git-chglog/git-chglog/releases/download/v0.10.0/git-chglog_0.10.0_linux_amd64.tar.gz | tar xzf -
                    ./git-chglog --output CHANGELOG.md $TAG_NAME
                    wget -q -O - https://github.com/cli/cli/releases/download/v1.6.2/gh_1.6.2_linux_amd64.tar.gz | tar xzf -
                    ./gh_1.6.2_linux_amd64/bin/gh auth status
                    ./gh_1.6.2_linux_amd64/bin/gh release -R $GIT_URL create $TAG_NAME --title $TAG_NAME -F CHANGELOG.md -d
                '''
            }
        }
    }

    post {
        always {
            script {
                env.GIT_COMMIT_MSG = sh(script: 'git log -1 --pretty=%B ${GIT_COMMIT}', returnStdout: true).trim()
                env.GIT_AUTHOR = sh(script: 'git log -1 --pretty=%cn ${GIT_COMMIT}', returnStdout: true).trim()
                discordSend(
                    title: "${env.JOB_NAME} Finished ${currentBuild.currentResult}",
                    description: "```${env.GIT_COMMIT_MSG}```",
                    link: "$env.RUN_DISPLAY_URL",
                    successful: currentBuild.resultIsBetterOrEqualTo("SUCCESS"),
                    thumbnail: 'https://wiki.jenkins-ci.org/download/attachments/2916393/headshot.png',
                    result: currentBuild.currentResult,
                    webhookURL: DISCORD_WEBHOOK_URL
                )
            }
        }
    }
}

def runKaniko() {
    sh '''#!/busybox/sh
    GIT_BRANCH_SLUG=$(echo $BRANCH_NAME | sed -e 's/\\//-/g')
    /kaniko/executor -f `pwd`/Dockerfile_release -c `pwd` --build-arg BINARY=interbtc-parachain \
        --destination=${REGISTRY}/${REPOSITORY}/${IMAGE}:${GIT_BRANCH_SLUG} \
        --destination=${REGISTRY}/${REPOSITORY}/${IMAGE}:${GIT_BRANCH_SLUG}-${GIT_COMMIT:0:6}
    '''
}

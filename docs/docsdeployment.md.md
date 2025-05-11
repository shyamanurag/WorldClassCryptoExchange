# WorldClass Crypto Exchange: Deployment Guide

## Overview

This document outlines the deployment process for the WorldClass Crypto Exchange platform. It covers infrastructure setup, security requirements, deployment pipeline, and monitoring configuration. The deployment follows a multi-environment approach with separate development, staging, and production environments.

## Prerequisites

### Infrastructure Requirements

* **Kubernetes Cluster**
  - Minimum 5 nodes per environment (dev/staging/prod)
  - Node specs: 16 CPU cores, 64GB RAM, 500GB SSD
  - Kubernetes version: 1.26+
  - Network policies enabled
  - Pod security policies enabled

* **Database Infrastructure**
  - PostgreSQL 15.0+ (for user data, transaction records)
  - TimescaleDB 2.10+ (for time-series market data)
  - Redis 7.0+ cluster (for caching and real-time processing)

* **Security Infrastructure**
  - Hardware Security Modules (HSMs) for key management
  - Multiple geographic locations for cold storage keys
  - WAF (Web Application Firewall) for API endpoints
  - DDoS protection services

* **Monitoring Infrastructure**
  - Prometheus and Grafana for metrics
  - ELK stack for log aggregation
  - Jaeger for distributed tracing
  - PagerDuty integration for alerts

### Access Requirements

* **Service Accounts**
  - CI/CD pipeline service account
  - Database administration service account
  - Kubernetes administration service account

* **Security Credentials**
  - SSL certificates for all domains
  - HSM access credentials
  - Cloud provider API credentials
  - Registry access credentials

## Deployment Architecture

```
[External Users] → [DDoS Protection] → [Load Balancer] → [WAF] → [API Gateway]
                                                              ↓
[Admin Users] → [Admin VPN] → [Load Balancer] → [WAF] → [Admin API]
                                                              ↓
                                                    [Kubernetes Cluster]
                                                              ↓
                                      ┌─────────────┬────────────┬─────────────┐
                                      ↓             ↓            ↓             ↓
                                [Trading Engine][Wallet Service][KYC Service][Other Microservices]
                                      ↓             ↓            ↓             ↓
                                      └─────────────┴────────────┴─────────────┘
                                                              ↓
                                      ┌─────────────┬────────────┬─────────────┐
                                      ↓             ↓            ↓             ↓
                                 [PostgreSQL]   [TimescaleDB]  [Redis]    [RocksDB]
```

## Infrastructure Setup

### 1. Kubernetes Cluster Setup

```bash
# Create Kubernetes cluster using Terraform
cd infrastructure/terraform
terraform init
terraform apply -var-file=environments/production.tfvars

# Configure kubectl
aws eks update-kubeconfig --name worldclass-crypto-production

# Verify cluster
kubectl get nodes
```

### 2. Database Provisioning

```bash
# Deploy PostgreSQL using Helm
helm repo add bitnami https://charts.bitnami.com/bitnami
helm install postgresql bitnami/postgresql \
  --namespace database \
  --create-namespace \
  --values infrastructure/helm/postgresql-values.yaml

# Deploy TimescaleDB
helm repo add timescale https://charts.timescale.com
helm install timescaledb timescale/timescaledb-single \
  --namespace database \
  --values infrastructure/helm/timescaledb-values.yaml

# Deploy Redis Cluster
helm install redis bitnami/redis \
  --namespace database \
  --values infrastructure/helm/redis-values.yaml
```

### 3. Secrets Management

```bash
# Initialize Vault
helm repo add hashicorp https://helm.releases.hashicorp.com
helm install vault hashicorp/vault \
  --namespace vault \
  --create-namespace \
  --values infrastructure/helm/vault-values.yaml

# Unseal Vault and configure
kubectl exec -n vault vault-0 -- vault operator init
kubectl exec -n vault vault-0 -- vault operator unseal

# Store critical secrets in Vault
kubectl exec -n vault vault-0 -- vault kv put secret/db/postgresql \
  username=admin \
  password=<generated-password>

kubectl exec -n vault vault-0 -- vault kv put secret/hsm \
  api-key=<hsm-api-key>
```

### 4. Service Mesh Configuration

```bash
# Install Istio
istioctl install --set profile=default -f infrastructure/istio/istio-config.yaml

# Enable Istio injection for namespaces
kubectl label namespace default istio-injection=enabled
kubectl label namespace trading istio-injection=enabled
kubectl label namespace wallet istio-injection=enabled
```

## Deployment Pipeline

### 1. CI/CD Pipeline Setup

The platform uses GitLab CI/CD for the deployment pipeline. The pipeline is defined in `.gitlab-ci.yml` and includes the following stages:

1. **Build**: Compile code and build container images
2. **Test**: Run unit tests, integration tests, and security tests
3. **Scan**: Perform security scanning and dependency analysis
4. **Publish**: Push container images to registry
5. **Deploy**: Deploy to Kubernetes cluster
6. **Verify**: Run post-deployment tests

```yaml
# Example .gitlab-ci.yml snippet
stages:
  - build
  - test
  - scan
  - publish
  - deploy
  - verify

variables:
  DOCKER_REGISTRY: registry.example.com
  KUBERNETES_NAMESPACE: ${CI_ENVIRONMENT_NAME}

build:
  stage: build
  script:
    - cargo build --release
    - docker build -t ${DOCKER_REGISTRY}/trading-engine:${CI_COMMIT_SHA} -f Dockerfile.trading-engine .
  artifacts:
    paths:
      - target/release/trading-engine

test:
  stage: test
  script:
    - cargo test
    - ./scripts/integration_tests.sh

security_scan:
  stage: scan
  script:
    - ./scripts/security_scan.sh
    - ./scripts/dependency_check.sh

publish:
  stage: publish
  script:
    - docker push ${DOCKER_REGISTRY}/trading-engine:${CI_COMMIT_SHA}
    - docker tag ${DOCKER_REGISTRY}/trading-engine:${CI_COMMIT_SHA} ${DOCKER_REGISTRY}/trading-engine:latest
    - docker push ${DOCKER_REGISTRY}/trading-engine:latest

deploy_staging:
  stage: deploy
  environment:
    name: staging
  script:
    - kubectl apply -f kubernetes/staging/
    - helm upgrade --install trading-engine ./charts/trading-engine --namespace ${KUBERNETES_NAMESPACE}
  only:
    - main

deploy_production:
  stage: deploy
  environment:
    name: production
  script:
    - kubectl apply -f kubernetes/production/
    - helm upgrade --install trading-engine ./charts/trading-engine --namespace ${KUBERNETES_NAMESPACE}
  only:
    - tags
  when: manual
```

### 2. Deployment Strategies

Different components use different deployment strategies based on their criticality:

* **Trading Engine**: Blue-Green deployment
* **Wallet Services**: Canary deployment
* **API Gateway**: Rolling update
* **Admin Dashboard**: Rolling update

Example blue-green deployment configuration:

```yaml
# kubernetes/trading-engine/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: trading-engine-blue
  namespace: trading
spec:
  replicas: 3
  selector:
    matchLabels:
      app: trading-engine
      deployment: blue
  template:
    metadata:
      labels:
        app: trading-engine
        deployment: blue
    spec:
      containers:
      - name: trading-engine
        image: registry.example.com/trading-engine:v1.2.3
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "4Gi"
            cpu: "2"
          limits:
            memory: "8Gi"
            cpu: "4"
        readinessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
```

### 3. Rollback Procedures

In case of deployment failures, the following rollback procedures are implemented:

1. **Automated Rollback**: If health checks fail after deployment, automatic rollback to the previous version
2. **Manual Rollback**: For more complex issues, manual rollback command can be executed

```bash
# Automated rollback is triggered by CI/CD pipeline

# Manual rollback command
kubectl rollout undo deployment/trading-engine -n trading

# For blue-green deployments
kubectl apply -f kubernetes/trading-engine/service-green.yaml  # Switch back to green
```

## Security Configuration

### 1. Network Security

```yaml
# Example NetworkPolicy
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: trading-engine-network-policy
  namespace: trading
spec:
  podSelector:
    matchLabels:
      app: trading-engine
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: api-gateway
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: database
    ports:
    - protocol: TCP
      port: 5432
```

### 2. Pod Security Policies

```yaml
# Example PodSecurityPolicy
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: trading-engine-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  volumes:
    - 'configMap'
    - 'emptyDir'
    - 'projected'
    - 'secret'
    - 'downwardAPI'
    - 'persistentVolumeClaim'
  hostNetwork: false
  hostIPC: false
  hostPID: false
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  supplementalGroups:
    rule: 'MustRunAs'
    ranges:
      - min: 1
        max: 65535
  fsGroup:
    rule: 'MustRunAs'
    ranges:
      - min: 1
        max: 65535
  readOnlyRootFilesystem: true
```

### 3. Secret Management

Secrets are managed through HashiCorp Vault and mounted into Kubernetes pods:

```yaml
# Example Kubernetes deployment with Vault integration
apiVersion: apps/v1
kind: Deployment
metadata:
  name: trading-engine
  namespace: trading
spec:
  replicas: 3
  selector:
    matchLabels:
      app: trading-engine
  template:
    metadata:
      labels:
        app: trading-engine
      annotations:
        vault.hashicorp.com/agent-inject: 'true'
        vault.hashicorp.com/agent-inject-secret-db-creds: 'secret/db/postgresql'
        vault.hashicorp.com/role: 'trading-engine'
    spec:
      serviceAccountName: trading-engine
      containers:
      - name: trading-engine
        image: registry.example.com/trading-engine:latest
        env:
        - name: DB_CREDS_PATH
          value: /vault/secrets/db-creds
```

## Monitoring and Alerting

### 1. Prometheus Configuration

```yaml
# prometheus/trading-engine-rules.yaml
groups:
- name: trading-engine
  rules:
  - alert: TradingEngineHighLatency
    expr: histogram_quantile(0.95, sum(rate(trading_engine_order_processing_duration_seconds_bucket[5m])) by (le)) > 0.1
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High latency in trading engine"
      description: "95th percentile of trading engine order processing is above 100ms for 5 minutes"

  - alert: TradingEngineErrorRate
    expr: sum(rate(trading_engine_errors_total[5m])) / sum(rate(trading_engine_requests_total[5m])) > 0.01
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate in trading engine"
      description: "Error rate is above 1% for 5 minutes"
```

### 2. Log Aggregation

```yaml
# Example Fluentd configuration for log collection
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: fluentd
  namespace: logging
spec:
  selector:
    matchLabels:
      app: fluentd
  template:
    metadata:
      labels:
        app: fluentd
    spec:
      serviceAccountName: fluentd
      containers:
      - name: fluentd
        image: fluent/fluentd-kubernetes-daemonset:v1.14-debian-elasticsearch7-1
        env:
          - name: FLUENT_ELASTICSEARCH_HOST
            value: "elasticsearch.logging"
          - name: FLUENT_ELASTICSEARCH_PORT
            value: "9200"
        volumeMounts:
        - name: varlog
          mountPath: /var/log
        - name: varlibdockercontainers
          mountPath: /var/lib/docker/containers
          readOnly: true
      volumes:
      - name: varlog
        hostPath:
          path: /var/log
      - name: varlibdockercontainers
        hostPath:
          path: /var/lib/docker/containers
```

### 3. Distributed Tracing

```yaml
# Example Jaeger configuration
apiVersion: jaegertracing.io/v1
kind: Jaeger
metadata:
  name: jaeger
  namespace: monitoring
spec:
  strategy: production
  storage:
    type: elasticsearch
    options:
      es:
        server-urls: http://elasticsearch:9200
```

## Cold Storage Deployment

The cold storage system requires special handling for deployment:

1. **Key Generation Ceremony**: Conducted offline with multiple authorized personnel
2. **Hardware Security Modules**: Initialized and configured in secure facilities
3. **Geographic Distribution**: Keys stored in multiple secure locations

```bash
# Example of offline key generation script (run in secure environment)
./scripts/cold_storage_key_generation.sh \
  --threshold 3 \
  --total-shares 5 \
  --output-dir /secure/cold-storage-keys

# Distribution of key shares to HSMs in different locations
for location in london tokyo singapore sydney zurich; do
  ./scripts/deploy_key_share.sh \
    --location $location \
    --share-file /secure/cold-storage-keys/share-$location.key \
    --hsm-id $location-hsm-001
done
```

## Environment-Specific Configurations

### Development Environment

```yaml
# development/values.yaml
environment: development
replicas:
  trading-engine: 1
  wallet-service: 1
  api-gateway: 1
resources:
  trading-engine:
    requests:
      cpu: 1
      memory: 2Gi
    limits:
      cpu: 2
      memory: 4Gi
database:
  host: postgres-dev.example.com
  name: worldclass_dev
```

### Staging Environment

```yaml
# staging/values.yaml
environment: staging
replicas:
  trading-engine: 2
  wallet-service: 2
  api-gateway: 2
resources:
  trading-engine:
    requests:
      cpu: 2
      memory: 4Gi
    limits:
      cpu: 4
      memory: 8Gi
database:
  host: postgres-staging.example.com
  name: worldclass_staging
```

### Production Environment

```yaml
# production/values.yaml
environment: production
replicas:
  trading-engine: 5
  wallet-service: 5
  api-gateway: 3
resources:
  trading-engine:
    requests:
      cpu: 4
      memory: 16Gi
    limits:
      cpu: 8
      memory: 32Gi
database:
  host: postgres-production.example.com
  name: worldclass_production
```

## Deployment Checklist

Before deploying to production, ensure the following checklist is completed:

### Pre-Deployment Checks

- [ ] All unit tests passing
- [ ] Integration tests passing
- [ ] Security scan completed with no critical issues
- [ ] Performance testing completed
- [ ] Dependency audit completed
- [ ] Documentation updated
- [ ] Rollback procedures tested
- [ ] Cold storage system tested
- [ ] Approval from security team
- [ ] Approval from compliance team

### Deployment Process

1. Announce maintenance window (if applicable)
2. Deploy database migrations
3. Deploy API gateway updates
4. Deploy microservices
5. Deploy trading engine
6. Deploy admin dashboard
7. Update load balancer configuration
8. Run health checks
9. Verify monitoring and alerting

### Post-Deployment Checks

- [ ] API endpoints returning correct responses
- [ ] Trading engine processing orders correctly
- [ ] Wallet service handling transactions
- [ ] Admin dashboard accessible
- [ ] Monitoring dashboards showing expected metrics
- [ ] Logs being collected properly
- [ ] Transaction testing completed
- [ ] Security audit of deployed services

## Disaster Recovery

### Backup Procedures

1. Database backups:
   - Full backup daily
   - Incremental backups hourly
   - Transaction log shipping continuous

2. Configuration backups:
   - Kubernetes configuration backed up daily
   - Vault configuration backed up after changes
   - Secrets backed up using secure procedure

### Recovery Procedures

1. **Database Recovery**:
```bash
# Restore PostgreSQL database
pg_restore -h postgres-production.example.com -U admin -d worldclass_production backup.sql

# Verify data integrity
./scripts/verify_data_integrity.sh
```

2. **Infrastructure Recovery**:
```bash
# Restore Kubernetes cluster from backup
terraform apply -var-file=environments/production.tfvars

# Apply configurations
kubectl apply -f kubernetes/production-backup/

# Verify cluster health
kubectl get nodes
kubectl get pods --all-namespaces
```

3. **Cold Storage Recovery**:
```bash
# Initiate cold storage recovery (requires multiple authorized personnel)
./scripts/cold_storage_recovery.sh \
  --threshold 3 \
  --shares-location /secure/recovery-shares/
```

## Compliance Documentation

Maintain the following documentation for compliance purposes:

1. Deployment logs
2. Access logs
3. Audit trails
4. Security scan reports
5. Penetration test reports
6. Change management records

## Contact Information

For deployment issues, contact:

- **DevOps Lead**: [devops-lead@example.com](mailto:devops-lead@example.com)
- **Security Team**: [security@example.com](mailto:security@example.com)
- **Database Admin**: [dba@example.com](mailto:dba@example.com)

Emergency contacts:
- **On-call Engineer**: +1-555-123-4567
- **Security Officer**: +1-555-987-6543

## Additional Resources

- [Internal Wiki: Deployment Procedures](https://wiki.example.com/deployment)
- [Knowledge Base: Common Issues](https://kb.example.com/deployment-issues)
- [Runbook: Emergency Procedures](https://runbooks.example.com/emergency)

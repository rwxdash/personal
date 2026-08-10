# Cloud Platform, IaC, Migration & Cost — Questions

The topics that come up in staff-level infrastructure interviews once the
protocol questions are done: how you run a platform, how you move things
without breaking them, and whether you know where the money goes. Q14 is the
cross-vendor migration question in its full form.

17 questions.

---

## Tier 1 — Recall

### Q1. Explain the difference between an IAM user, a role, and a policy. What does `AssumeRole` actually do?
*Tags: iam, aws, identity*

### Q2. How does a pod get AWS credentials without a static access key? Explain IRSA / Workload Identity.
*Tags: iam, kubernetes, identity*

### Q3. What is envelope encryption, and what is a KMS data key?
*Tags: kms, encryption*

### Q4. What is Terraform state, why does it need locking, and what happens when it drifts from reality?
*Tags: terraform, iac*

### Q5. Define RPO and RTO. Give the four standard DR tiers and their rough cost/RTO profile.
*Tags: dr, resilience*

---

## Tier 2 — Explain / compare

### Q6. Compare Terraform and AWS CDK/CloudFormation. When would you choose each, and what does each get wrong?
*Tags: iac, terraform, cdk* · *[on your resume — you've run both]*

### Q7. How do you structure Terraform for a large organisation? Discuss state splitting, module design, and blast radius.
*Tags: terraform, scale, organisation*

### Q8. Compare blue-green, canary, and rolling deployments. How do you roll back a change that altered data?
*Tags: deploys, rollback*

### Q9. What is the confused deputy problem in a cross-account context, and what is an external ID for?
*Tags: iam, security, saas*

### Q10. Design a multi-account strategy for an organisation of 200 engineers. What are the account boundaries and the guardrails?
*Tags: aws, organisation, governance*

### Q11. Where does a typical cloud bill actually go? Give the levers in order of impact, and say which ones cost you reliability.
*Tags: cost, finops*

### Q12. How do you decide between a managed service and self-hosting? Give a framework and apply it to Kafka.
*Tags: build-vs-buy, decision-making*

### Q13. A cloud region "goes down". What actually fails, and what surprises teams who thought they were multi-AZ?
*Tags: dr, failure-modes, control-plane*

---

## Tier 3 — Scenario / debug

### Q14. Your company is moving from AWS to GCP over 18 months. You own the plan. Walk me through the whole programme — sequencing, connectivity, identity, data, and how you avoid a big-bang cutover.
*Tags: migration, programme, multi-cloud* · *[asked in your interviews]*

### Q15. You've acquired a company. They have their own AWS account, their own Kubernetes clusters, their own on-call, and 20+ microservices that must talk to yours. Plan the first 90 days.
*Tags: acquisition, integration* · *[you have done this]*

### Q16. Your cloud bill grew 40% in a quarter with flat traffic. Find the money.
*Tags: cost, investigation, method*

### Q17. You need to size a fleet for a workload you have never run in production. How do you do it, and how do you avoid being wrong in the expensive direction?
*Tags: capacity-planning, method*

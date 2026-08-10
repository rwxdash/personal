# Cloud Platform, IaC, Migration & Cost — Answers

Provider limits, pricing, and service capabilities change. Treat specific
numbers here as order-of-magnitude and verify current documentation before
quoting them as exact.

---

## Tier 1 — Recall

### A1. Users, roles, policies, AssumeRole

**Answer.**
- **User** — a long-lived identity with permanent credentials (a password,
  or an access key pair). The credentials don't expire, which is exactly the
  problem: leaked keys stay valid until someone notices.
- **Role** — an identity with **no permanent credentials**, defined by two
  policies: a **trust policy** (who may assume it) and **permission
  policies** (what it may do once assumed). Roles are the mechanism for
  every modern access pattern: EC2 instance profiles, Lambda execution
  roles, cross-account access, federated SSO, IRSA.
- **Policy** — a JSON document of statements (`Effect`, `Action`,
  `Resource`, optional `Condition`). Identity policies attach to a
  principal; **resource policies** (S3 bucket policies, KMS key policies)
  attach to the resource and can grant access to principals in other
  accounts. Explicit `Deny` always wins over any `Allow`.

**`AssumeRole`** calls STS and gets back a **temporary credential set** —
access key, secret key, and a session token — with an expiry (15 minutes to
12 hours depending on configuration). Behind it, STS checks that your
principal is permitted by the role's trust policy *and* that your own
identity policy allows `sts:AssumeRole` on that role. Both sides must agree,
which is what makes cross-account access safe to grant.

Two important consequences:
- The resulting session's permissions are the **intersection** of the role's
  policies and any **session policy** you pass. That's how you do
  least-privilege delegation — hand out a role but scope the session down
  further.
- Because credentials expire, a leak has a bounded blast radius. This is the
  single strongest argument for eliminating IAM users entirely in favour of
  SSO/federation plus roles, and it's the answer to "how do you manage
  credentials at scale": you don't — you make them short-lived and issued on
  demand.

**Weak answers miss.** That a role has *two* policies and both sides must
permit the assumption, and the session-policy intersection.

**Follow-ups to expect.**
- How do you audit who did what when everyone assumes roles? (CloudTrail
  records the assumed-role session with the source identity and the session
  name — which is why setting a meaningful `RoleSessionName` matters, and
  why `sts:SourceIdentity` exists.)
- What's a permission boundary? (A policy that caps the *maximum*
  permissions an identity can have, regardless of what's attached to it. The
  mechanism for safely letting teams create their own roles without being
  able to privilege-escalate.)

---

### A2. IRSA / Workload Identity

**Answer.**
The problem: a pod needs cloud permissions. The bad old answers were
(a) static access keys in a Secret — long-lived, leakable, hard to rotate —
or (b) the node's instance profile, which gives *every pod on the node* the
same permissions, so least privilege is impossible.

**IRSA (IAM Roles for Service Accounts)** solves it with OIDC federation:

1. The cluster runs an **OIDC identity provider**; its public JWKS endpoint
   is registered as a trusted identity provider in IAM.
2. A Kubernetes **ServiceAccount** is annotated with a role ARN.
3. When a pod using that ServiceAccount starts, a mutating admission webhook
   projects a **short-lived, audience-scoped ServiceAccount token** (a
   signed JWT) into the pod as a file, and sets `AWS_ROLE_ARN` and
   `AWS_WEB_IDENTITY_TOKEN_FILE`.
4. The AWS SDK sees those variables and calls
   `sts:AssumeRoleWithWebIdentity`, presenting the JWT.
5. STS validates the signature against the cluster's JWKS, checks the
   audience and the `sub` claim (`system:serviceaccount:<ns>:<name>`)
   against the role's trust policy, and issues temporary credentials.

What this gives you: **per-pod, least-privilege cloud identity with no
static secrets**, credentials that rotate automatically, and a trust
relationship expressed as "this specific ServiceAccount in this specific
namespace in this specific cluster."

GKE **Workload Identity** is the same shape — a Kubernetes ServiceAccount
maps to a Google service account through a workload identity pool, and the
metadata server issues tokens. **EKS Pod Identity** is a newer AWS variant
that removes the per-cluster OIDC provider setup and uses an agent instead;
simpler to operate, slightly different trust model — worth knowing both
exist.

The detail that matters for security: **the trust policy condition must pin
the `sub` claim to the exact namespace and ServiceAccount name.** A trust
policy that only checks the OIDC issuer means *any* ServiceAccount in the
cluster can assume the role — a very common misconfiguration that quietly
grants cluster-wide access to a role you thought was scoped.

**Weak answers miss.** The `sub` claim pinning, and that this is standard
OIDC federation rather than a bespoke mechanism.

**Follow-ups to expect.**
- What's the equivalent on bare metal? (No cloud metadata service, so
  SPIFFE/SPIRE or Vault's Kubernetes auth method — same idea, the
  ServiceAccount token is the attestation.)
- What breaks if the OIDC endpoint is unreachable? (STS can cache JWKS, but
  new assumptions can fail. It's a dependency worth knowing about.)

---

### A3. Envelope encryption

**Answer.**
Rather than encrypting data directly with a master key, you use two layers:

1. Ask KMS to **generate a data key**. It returns the key twice: once in
   **plaintext** and once **encrypted** under the customer master key (CMK),
   which never leaves KMS.
2. Encrypt your data locally with the plaintext data key (AES-GCM), then
   **discard the plaintext key from memory**.
3. Store the **encrypted data key alongside the ciphertext**.
4. To decrypt: send the encrypted data key to KMS, get the plaintext back,
   decrypt locally, discard.

Why this design:
- **The master key never leaves the HSM**, so it can't be exfiltrated.
- **Bulk data never transits KMS.** KMS has request size limits (a few KB)
  and rate limits; you cannot send a 10 GB object through it. Envelope
  encryption means KMS only ever handles a 256-bit key.
- **Performance**: local symmetric encryption with AES-NI is nearly free.
  One KMS call per object (or per batch) instead of per byte.
- **Key rotation is cheap**: rotating the CMK means re-encrypting the small
  data keys, not the terabytes of data. (Note: AWS's automatic CMK rotation
  keeps old key material for decrypting existing data keys, so nothing needs
  re-encrypting at all — the tradeoff is that old data is still protected by
  older material.)
- **Caching**: you can reuse a data key across many objects for a bounded
  time to reduce KMS calls, trading some blast radius for cost and latency.

Worth adding: **encryption context** — additional authenticated data passed
to KMS on encrypt and required on decrypt. It binds the data key to a
context (e.g. `{"tenant": "acme"}`), so a data key stolen from one context
can't be used in another, and it appears in CloudTrail, which makes audit
logs meaningful.

**Weak answers miss.** The KMS size/rate limit as the practical motivation,
and encryption context.

**Follow-ups to expect.**
- What does "encrypted at rest" actually protect against? (Physical media
  theft and improper disposal, and — with per-tenant keys — a cross-tenant
  data exposure. It does **not** protect against a compromised application
  with decrypt permission, which is how most real breaches happen. Being
  honest about this is a good signal.)
- Who can decrypt? (Anyone with `kms:Decrypt` on the CMK **and** access to
  the ciphertext. Key policies are the real access control; a bucket policy
  alone isn't enough. Separating those two permissions is a genuine defence.)

---

### A4. Terraform state

**Answer.**
State is Terraform's record of the mapping between resources in your
configuration and real objects in the provider, plus their last-known
attributes. It exists because Terraform must know that
`aws_instance.web` corresponds to `i-0abc123` — the provider APIs have no
concept of your resource names.

**Why locking**: `terraform apply` reads state, computes a plan, makes
changes, and writes state back. Two concurrent applies interleave those
steps and can produce a state file that doesn't reflect reality — resources
created but not recorded (which then get created again, or block a future
apply), or recorded but not created. So the backend takes a lock (DynamoDB
for S3 backends, native locking in Terraform Cloud/GCS). Running Terraform
without locking in a team is a matter of time, not risk.

**Drift** is when reality diverges from state — someone changed a resource
in the console, another tool modified it, or the provider changed a default.
Consequences:
- `terraform plan` shows unexpected changes, and the next apply **reverts**
  the manual change. Usually correct, occasionally catastrophic (reverting an
  emergency fix made during an incident).
- If a resource was **deleted** outside Terraform, the plan tries to
  update a nonexistent thing and errors, or plans to recreate it.
- `terraform refresh` (now folded into plan) reconciles state with reality.
  `terraform import` brings an unmanaged resource under management.
  `terraform state rm` forgets a resource without destroying it.

State also contains **secrets in plaintext** — database passwords,
generated keys, certificate private keys. So the backend must be encrypted,
access-controlled, and versioned. Treating a state file as non-sensitive is
a common and serious mistake.

The practices that follow: remote backend with locking and versioning;
nobody runs apply from a laptop against production (CI/CD with a plan
approval step); manual console changes to managed resources are a policy
violation, detected by scheduled drift detection; and state file versioning
so you can recover from a corrupted write.

**Weak answers miss.** That state contains secrets, and that drift detection
should be a scheduled job rather than something you discover during an
apply.

**Follow-ups to expect.**
- Someone deleted the state file. What now? (Import everything, or recreate.
  Which is why versioned backends exist. Also a good argument for many small
  states — the blast radius of losing one is smaller.)
- What is `create_before_destroy` and when do you need it? (Lifecycle rule to
  build the replacement before removing the old one — essential for anything
  in a serving path, and it requires name uniqueness, which is why you use
  `name_prefix`.)

---

### A5. RPO, RTO, and DR tiers

**Answer.**
- **RPO (Recovery Point Objective)** — how much *data* you can afford to
  lose, measured in time. RPO of 5 minutes means the last 5 minutes of
  writes may be gone. It's determined by your replication/backup mechanism.
- **RTO (Recovery Time Objective)** — how long recovery may take. It's
  determined by how much has to be built, restored, or promoted.

They're independent, and confusing them is common. Async replication gives
you a low RTO and a nonzero RPO; nightly backups give you a 24-hour RPO and
a potentially long RTO.

**The four tiers:**

| Tier | RPO | RTO | Steady-state cost | What it is |
| --- | --- | --- | --- | --- |
| **Backup & restore** | Hours | Hours–days | Very low (storage only) | Backups in another region; rebuild everything on demand |
| **Pilot light** | Minutes | Tens of minutes–hours | Low | Data replicated continuously; minimal core infrastructure running; compute scaled from zero on failover |
| **Warm standby** | Seconds–minutes | Minutes | Medium | A scaled-down but *running* copy of the full stack; scale up and shift traffic |
| **Active-active / hot** | ~Zero | Seconds | High (2x+) | Both regions serving; failover is traffic shifting |

The judgements worth stating:

- **Cost rises steeply and RTO falls; pick per workload, not per company.**
  A tiered approach — active-active for the customer-facing API, warm
  standby for internal services, backup-and-restore for analytics — is
  almost always right, and "everything active-active" is a sign nobody did
  the analysis.
- **The RTO you have is the one you've measured**, not the one in the
  document. Pilot light in particular hides enormous RTO risk: quotas in the
  DR region you've never requested, AMIs not replicated, a bootstrapping
  dependency on the failed region, DNS TTLs, and — the classic — the whole
  region's worth of instances not being available on demand because everyone
  else is failing over too.
- **Active-active is the only configuration continuously proven**, because
  it's exercised by normal traffic. Everything else needs deliberate
  **failover drills**, and a DR plan that has never been executed is a
  hypothesis. This is the single most important thing to say.
- The **data layer determines the floor** on both numbers (topic 06, A9 and
  A16). No amount of compute preparation gives you an RPO better than your
  replication mode.

**Weak answers miss.** That RTO must be measured by drilling, and the
capacity-availability problem during a real regional event.

**Follow-ups to expect.**
- What's your RPO with async cross-region replication? (Equal to the
  replication lag at the moment of failure — which is largest under load,
  i.e. exactly when you're likely to fail. Monitor and alert on it as an RPO
  metric, not just a performance metric.)
- How do you fail *back*? (Often harder than failing over, because the
  original region's data is now stale and diverged. Reverse replication and
  a planned cutover — topic 06, A16. Teams plan the failover and not the
  return, and then run in DR for months.)

---

## Tier 2 — Explain / compare

### A6. Terraform vs CDK/CloudFormation

**Answer.**
**Terraform** — declarative HCL, provider-based, external state.
- **Multi-cloud and multi-provider.** One tool for AWS, GCP, Cloudflare,
  Datadog, GitHub, Postgres roles, Kubernetes. In practice most
  organisations need several of these, and having one workflow is a real
  benefit.
- **`terraform plan` is genuinely good** — an explicit, reviewable diff
  before anything changes. This is the feature people miss most when they
  leave.
- Mature module ecosystem, and state you can inspect, import, and surgically
  repair.
- What it gets wrong: **HCL is not a programming language.** Loops
  (`for_each`), conditionals (`count` with ternaries), and dynamic blocks
  are awkward, and complex logic becomes unreadable. State is a real
  artifact you must manage, secure, and occasionally repair by hand.
  Provider coverage of brand-new AWS features lags. And the CDKTF/OpenTofu
  licensing history is now part of the decision.

**AWS CDK** — real programming languages (TypeScript, Python, Go, Java)
synthesising CloudFormation.
- **Real abstraction.** Classes, inheritance, unit tests, IDE completion,
  and shared libraries with types. For a platform team building reusable
  constructs, this is a substantial productivity difference.
- **L2/L3 constructs** encode sensible defaults — a `Vpc` construct
  generates subnets, route tables, NAT gateways, and IGW from a handful of
  parameters. Enormous boilerplate reduction.
- **No state file to manage.** CloudFormation tracks it server-side, with
  automatic rollback on failed updates.
- What it gets wrong: **AWS only** (CDK for Terraform exists but is a
  different product). **CloudFormation's limitations become yours** —
  historically slow updates, stack resource limits, and drift that is hard
  to reconcile. **Rollback can get stuck** (`UPDATE_ROLLBACK_FAILED`), which
  is genuinely painful under pressure. The plan equivalent (`cdk diff`) is
  weaker than Terraform's because it diffs CloudFormation templates rather
  than resolved resource state. And the abstraction cuts both ways: a
  one-line construct change can generate a large, surprising diff, so you
  must read the synthesised template for anything consequential.

**How I'd choose:**
- **Multi-cloud, or a lot of non-cloud providers** → Terraform. There's no
  contest.
- **AWS-only, with a platform team building reusable abstractions for
  application teams** → CDK. The typed-construct library is the winning
  argument.
- **Team skills matter more than the tool.** A team fluent in TypeScript
  will produce better infrastructure in CDK than in HCL they resent, and
  vice versa.
- **Don't mix them for the same resources.** Mixing for *different* layers
  (Terraform for accounts/networking/foundations, CDK for application
  stacks) is a defensible and common split, and the boundary should be along
  lifecycle lines: things that change rarely vs things that change with the
  app.

Having migrated a CloudFormation/CDK estate as part of an acquisition
integration, the point I'd make from experience: the tool matters far less
than **whether the definitions are complete**. Half-managed infrastructure —
some resources in code, some clicked into existence — is worse than either
tool, and eliminating drift is the actual work.

**Weak answers miss.** The plan-quality difference, the stuck-rollback
failure mode, and that the real cost is partially-managed infrastructure
rather than syntax.

**Follow-ups to expect.**
- Pulumi? (Real languages like CDK, provider model like Terraform, its own
  state service. Genuinely good; the adoption question is ecosystem size and
  vendor dependence.)
- How do you test infrastructure code? (Static: `validate`, `tflint`,
  `checkov`/`tfsec` for policy. Unit: CDK assertions / Terratest plan
  assertions. Integration: apply to an ephemeral environment and assert.
  Policy-as-code (OPA/Sentinel) on the plan output is the highest-value
  layer, because it enforces org rules rather than syntax.)

---

### A7. Terraform at organisational scale

**Answer.**
The single organising principle: **state boundaries are blast-radius
boundaries.** A monolithic state means every apply risks every resource,
plans take many minutes, the lock serialises the whole organisation, and
losing the state file is an extinction event.

**How I'd split state**, in order of importance:

1. **By lifecycle / rate of change.** Foundations (accounts, VPCs, transit
   gateways, IAM baseline) change monthly; application infrastructure
   changes daily. Separate states so a routine app change can't touch the
   VPC.
2. **By environment.** prod / staging / dev in entirely separate states and
   ideally separate accounts. Never use Terraform *workspaces* for
   environments — they share a configuration and a backend, so a mistake
   crosses the boundary, and the difference between environments is
   invisible in the code. Separate directories with separate backends make
   the boundary explicit and reviewable.
3. **By team ownership.** A team should be able to apply their own state
   without coordinating.
4. **By region**, for anything regional, so a region's changes are isolated.

**How the layers connect** — this is where it usually goes wrong:
- **Remote state data sources** (`terraform_remote_state`) create a tight
  coupling: the consumer breaks if the producer's outputs change, and you
  get an implicit dependency graph nobody can see.
- Better: publish stable identifiers to a **registry** — SSM Parameter
  Store, or resource tags queried by data sources. The consumer looks up
  "the prod VPC" by tag rather than by reaching into another state file.
  Looser coupling, and it survives the producer being refactored or replaced.

**Module design:**
- **Two kinds of modules.** *Primitive* modules wrap a resource with your
  conventions (naming, tagging, encryption defaults). *Composite* modules
  assemble primitives into a pattern ("a standard service": ECS service +
  ALB target group + log group + alarms + IAM role). Application teams
  consume composites; the platform team maintains both.
- **Version modules and pin them.** Git tags or a private registry. An
  unpinned module means an upstream change alters every consumer's next
  plan — which is how you get a surprise destroy.
- **Keep modules shallow.** Deeply nested modules make plans unreadable and
  variables have to be threaded through every layer. Two levels is usually
  enough.
- **Don't over-abstract early.** A module with 40 optional variables is
  worse than three explicit configurations; abstraction should follow
  demonstrated repetition.

**Guardrails:**
- **Policy as code** on the plan — OPA/Conftest/Sentinel — enforcing
  encryption, tagging, no public S3, no 0.0.0.0/0 ingress, allowed instance
  types and regions. Cheaper and more reliable than review.
- **CI/CD only**: plan on PR (posted as a comment), apply on merge, with the
  apply running under a role humans don't hold. Nobody applies from a laptop
  to production.
- **Mandatory tags** for cost attribution (A11 and A16), enforced by policy
  and by AWS tag policies.
- **Scheduled drift detection** — a nightly plan across all states that
  alerts on non-empty diffs.
- **`prevent_destroy`** on stateful resources, and a review requirement for
  any plan containing a destroy or replace of a database, bucket, or
  anything holding data. Reading "1 to destroy" in a 400-line plan is how
  data gets lost; make the tooling shout about it.

**The tension worth naming:** more states means smaller blast radius and
faster plans, but more cross-state dependencies and more operational
surface. The right granularity is the one where an average change touches
exactly one state and a team can move without coordinating — and I'd tune it
by observing how often applies need to span states.

**Weak answers miss.** Not using workspaces for environments, and the
registry-instead-of-remote-state pattern for cross-layer coupling.

**Follow-ups to expect.**
- How do you handle a change that spans states? (Order it explicitly:
  additive change in the lower layer first, then the consumer, then cleanup.
  Same expand/contract discipline as a schema migration — topic 06, A15.)
- Terragrunt? (Solves the DRY-backend-configuration and dependency-ordering
  problems that plain Terraform leaves to you. Useful at scale; adds a tool
  and a layer of indirection. Native support has closed some of the gap.)

---

### A8. Blue-green, canary, rolling — and rolling back data

**Answer.**
**Rolling** — replace instances incrementally within one environment.
Cheapest (no duplicate capacity), but rollback is another rolling operation
(slow), both versions serve simultaneously (so they must be compatible), and
every user is exposed to the new version from the first replaced instance.
Controls *capacity* during the change, not *risk*.

**Blue-green** — build a complete parallel environment, verify it, switch
traffic at once, keep the old one for a while.
- Pro: **instant rollback** — flip traffic back. Full verification before
  any user traffic. Only one version serves at a time.
- Con: **2x capacity** during the transition; the switch is all-or-nothing
  so every user is exposed simultaneously; long-lived connections are cut at
  the flip; and shared state (the database) is *not* duplicated, so both
  environments must work against the same schema.

**Canary** — route a small percentage to the new version, measure, expand.
- Pro: **bounded blast radius** — a bad release affects 1% of users; and you
  get real production signal (error rate, latency, business metrics) before
  full exposure. With automated analysis (Argo Rollouts, Flagger) the
  promotion or rollback decision is made by data, not by a human's nerve.
- Con: slowest; both versions run simultaneously (compatibility again);
  needs traffic-splitting infrastructure and good per-version metrics; and a
  low-traffic service can't produce statistically meaningful canary signal.

**They're complementary, not alternatives**: rolling is a capacity strategy,
canary is a risk strategy, blue-green is a rollback strategy. Canary on top
of rolling is the common production answer.

**Rolling back a change that altered data** — the hard part, and the point
of the question. **You usually can't.** Traffic switching is instant and
reversible; data changes are neither.

The discipline that makes it possible:

1. **Expand/contract (parallel change).** Never a destructive schema change
   in the same release as the code that needs it (topic 06, A15):
   - *Expand*: add the new column/table, nullable, with a default. Deploy.
     Old code ignores it.
   - *Migrate*: dual-write to old and new; backfill in batches.
   - *Switch*: deploy code that reads from the new. Old code still works.
   - *Contract*: after a bake period and confidence, remove the old.
   Each step is independently deployable and independently reversible,
   because at no point is the old code unable to function. The cost is
   **four deploys instead of one**, and that cost is the price of
   reversibility.
2. **Backwards-compatible changes only** for the window in which both
   versions run. This is a hard constraint of rolling and canary alike, and
   it's frequently violated by accident (a renamed field, a changed enum, a
   removed API parameter).
3. **Feature flags** to separate *deploying* code from *activating*
   behaviour. Rolling back then means flipping a flag — seconds, no
   redeploy. This is the single highest-leverage practice for fast rollback,
   and it converts most rollbacks into a config change.
4. **For genuinely irreversible changes** (a destructive migration, a data
   transformation): the rollback plan is **forward-fix plus restore**, which
   means a tested backup, a known RPO, and an explicit "point of no return"
   in the runbook that everyone acknowledges before crossing.

**The sentence to say**: rollback is a property you *design in*, not an
operation you perform. If the release wasn't built to be reversible, there
is no rollback button, and that decision was made at design time, not at
3am.

**Weak answers miss.** Feature flags as the fastest rollback mechanism, and
that expand/contract's cost is extra deploys — being explicit about the cost
is what makes the recommendation credible.

**Follow-ups to expect.**
- How do you canary a stateful service or a data pipeline? (Shadow traffic /
  dual-write to the new version and compare outputs without serving them.
  You can't split users when there's shared state, so you compare instead.)
- What about rolling back a config change? (Same discipline — config should
  go through the same progressive rollout as code, which is the lesson from
  topic 09, A17.)

---

### A9. Confused deputy and external ID

**Answer.**
**The confused deputy problem**: a privileged component is tricked into
misusing its authority on behalf of someone who shouldn't have it. The
component isn't compromised — it's *confused* about who it's acting for.

**The concrete cross-account version.** A SaaS vendor (say a monitoring or
cost-management product) needs read access to your AWS account. The standard
pattern: you create a role in your account whose trust policy allows the
vendor's AWS account to assume it.

The attack: the vendor's account is the deputy. Suppose an attacker signs up
for the same SaaS product and, during setup, tells it "my role ARN is
`arn:aws:iam::<YOUR-ACCOUNT>:role/VendorAccess`" — your ARN, which they may
have discovered or guessed. The vendor's service dutifully calls
`AssumeRole` on that ARN. Your trust policy says "the vendor's account may
assume this role." The vendor's account *is* calling. It succeeds. The
attacker now reads your account through the vendor's UI.

Nothing was compromised. The vendor's credentials were used exactly as
permitted. Your trust policy simply couldn't distinguish "the vendor acting
for me" from "the vendor acting for someone else."

**External ID** fixes it. You generate a secret, unique value and give it to
the vendor at setup; your trust policy adds:

```json
"Condition": { "StringEquals": { "sts:ExternalId": "your-unique-secret" } }
```

Now the vendor must pass that exact value in the `AssumeRole` call. The
attacker doesn't know it, so their attempt fails even though the vendor's
account is authorised. It binds the assumption to a specific *customer
relationship*, not just to the vendor's identity.

Two operational rules: the external ID must be **generated by the vendor or
be unpredictable** (a customer-chosen "acme-corp" defeats the purpose), and
it must be **unique per customer** (a shared value means one customer can
attack another). Any competent vendor generates it for you; if a vendor lets
you pick it, that's a finding.

Related mechanisms worth naming: **`aws:SourceAccount` and `aws:SourceArn`**
conditions on *resource* policies, which solve the same class of problem for
service-to-service access (an S3 bucket policy allowing CloudTrail to write
should pin the source account, or another account's trail could write into
your bucket). And **VPC endpoint policies / `aws:PrincipalOrgID`** for
constraining access to your organisation.

**Weak answers miss.** Explaining *why* the vendor's account being
authorised isn't sufficient — the answer must include the second-customer
attack, or it's just a definition.

**Follow-ups to expect.**
- Is an external ID a secret? (Semi. It's not a credential on its own —
  knowing it grants nothing without the vendor's account — but it must be
  unpredictable. Treat it as sensitive.)
- Where else does the confused deputy show up? (SSRF against the instance
  metadata service is the same shape — the deputy is the metadata service,
  and IMDSv2's session-token requirement is the fix. Also CSRF in web apps.
  Naming the general pattern is the strong answer.)

---

### A10. Multi-account strategy for 200 engineers

**Answer.**
The premise: **the account is AWS's strongest isolation boundary.** Not IAM
policies, not tags, not VPCs — the account. Service quotas are per account,
blast radius of a misconfigured policy is per account, billing is per
account, and a compromised credential is scoped to an account. So the design
question is where to draw those boundaries.

**Account boundaries I'd use:**

1. **Per workload, per environment.** `payments-prod`, `payments-staging`,
   `payments-dev`. This is the primary axis. Why: prod isolation is
   absolute — no IAM mistake in dev can touch prod; quotas don't compete
   between environments (a load test in staging can't exhaust prod's ENI
   limit); and cost attribution is exact without any tagging discipline.
2. **Shared services accounts**: networking (Transit Gateway, DX, Route 53
   zones), security/logging (CloudTrail, Config, GuardDuty aggregation —
   with logs written to an account that workload teams cannot delete from),
   shared tooling (CI, artifact registries), and identity.
3. **Sandbox accounts** per engineer or per team, with a hard spending cap
   and automatic cleanup. This is what stops people experimenting in shared
   environments, and it's high value for morale as well as safety.
4. **Organisation management account** — used for nothing except
   Organizations, SCPs, and consolidated billing. No workloads. Ever.

For 200 engineers and, say, 40–60 services, that's on the order of 150–250
accounts. That number alarms people, and the answer is that accounts are
free and unmanaged accounts are the problem — which is why the guardrails
below are the actual work.

**Guardrails:**

- **Organizations with OUs** mirroring the structure (Prod, NonProd,
  Sandbox, Security, Infrastructure, Suspended).
- **SCPs** applied at the OU level. These are the controls that cannot be
  overridden by an account admin, so they're for the things that must be
  absolutely true:
  - Deny disabling CloudTrail, Config, or GuardDuty.
  - Deny leaving the organisation.
  - Deny use of regions you don't operate in (large reduction in attack
    surface and in accidental spend).
  - Deny deleting or modifying the org-managed IAM roles and the log
    destination.
  - Deny creating IAM users / long-lived access keys (forcing SSO + roles —
    A1).
  - Require encryption / deny public S3 at the org level.
  SCPs are **deny-only in practice** — they set the ceiling, they don't
  grant anything.
- **Account vending**: a pipeline (Control Tower / a custom factory) that
  creates an account with baseline networking, roles, logging, budget
  alarms, and guardrails already in place. Manual account creation must be
  impossible, or you get accounts outside the controls — which is the most
  common failure of a multi-account strategy.
- **Centralised identity**: IAM Identity Center / SSO with permission sets
  mapped to groups from your IdP. No IAM users anywhere. Break-glass roles
  exist, are alarmed on use, and require MFA.
- **Centralised networking**: VPCs created by the platform, attached to the
  Transit Gateway, with IPAM-allocated non-overlapping CIDRs (topic 04, A10
  — this is where you prevent that entire problem class). Consider sharing
  subnets via RAM so workload accounts don't manage networking at all.
- **Centralised logging**: CloudTrail org trail, Config, VPC flow logs, and
  application logs written to an account workload teams can't delete from.
  Immutability is the point.
- **Cost**: consolidated billing, per-account budgets with alerts, and
  mandatory tagging enforced by tag policies for sub-account attribution.

**Costs of this approach, stated honestly:** cross-account access adds
complexity to everything (CI/CD needs roles in every account, cross-account
resource sharing needs RAM or resource policies); some resources are
awkward to share; per-account overhead (interface endpoints, NAT gateways)
multiplies fixed costs, which is a real number at 200 accounts and an
argument for centralised egress; and the platform team owns a fleet of
accounts that must all be kept current. The mitigation for all of it is
automation — if account setup isn't fully automated, the strategy collapses
into inconsistency within a year.

**Weak answers miss.** SCPs as an uncircumventable ceiling distinct from
IAM, the account-vending requirement, and the per-account fixed-cost
multiplication.

**Follow-ups to expect.**
- Why not one account with separate VPCs? (Quotas are shared — one team's
  load test exhausts everyone's limits; IAM blast radius is shared; a
  compromised credential reaches everything; cost attribution depends on
  perfect tagging discipline that never survives contact with reality.)
- How do you do CI/CD across 200 accounts? (A deployment role in every
  account with a trust policy scoped to the CI account's role, provisioned
  by the account factory — never hand-created, or you'll have 200 subtly
  different trust policies.)

---

### A11. Where the cloud bill goes

**Answer.**
Typical distribution for a service-oriented company (order of magnitude, and
it varies a lot by workload):
1. **Compute** — 40–60%. EC2/EKS/Fargate/Lambda.
2. **Data transfer** — 10–25%. Internet egress, cross-region, and
   **cross-AZ** (topic 04, A13), which is the invisible one.
3. **Storage** — 10–20%. EBS (including unattached volumes and forgotten
   snapshots), S3 (including old versions and incomplete multipart uploads),
   and backups.
4. **Managed services** — 10–30%. RDS, MSK, OpenSearch, and observability
   vendors, which for a monitoring-heavy company can be a top-three line
   item on its own.
5. **Everything else** — NAT gateways, load balancers, KMS calls, VPC
   endpoints. Individually small, collectively surprising.

**Levers, in order of impact:**

1. **Commitment discounts** — Savings Plans / Reserved Instances / CUDs.
   Typically 30–70% off on-demand for a 1–3 year commitment. Zero
   engineering effort and no reliability cost. It is almost always the
   largest single win and the first thing to do, and the only risk is
   over-committing — so commit to your *baseline*, not your peak, and layer
   on-demand above it.
2. **Right-sizing** — matching instance size and Kubernetes requests to
   actual usage. Requires measurement (VPA recommendations, utilisation
   histograms) and it's continuous, not one-off. Typically 20–40% of compute
   in an unexamined estate.
3. **Eliminating waste** — unattached EBS volumes, old snapshots, idle load
   balancers, forgotten dev environments, orphaned Elastic IPs, S3 without
   lifecycle policies. Pure profit; find it once with a scanner and then
   automate the detection.
4. **Storage lifecycle** — S3 Intelligent-Tiering or explicit lifecycle to
   IA/Glacier; gp2 → gp3 (cheaper and independently tunable IOPS);
   log/metric retention tiers (topic 09, A12).
5. **Data transfer reduction** — VPC endpoints instead of NAT for AWS
   services, topology-aware routing to keep traffic same-AZ, a CDN in front
   of internet egress, compression. Requires architectural work, which is
   why it's below the easy wins despite being large.
6. **Spot / preemptible** — 60–90% off for interruptible workloads. Great
   for batch, CI, and stateless services with good disruption handling.
7. **Architectural change** — a more efficient data store, better caching,
   fewer round trips. Highest ceiling, highest effort, and the one that
   requires actual engineering time.

**Which levers cost reliability** — the part that separates a thoughtful
answer:
- **Spot** trades cost for interruption. Fine for batch; for serving
  traffic it requires handling 2-minute termination notices, diversified
  instance types, and an on-demand baseline. It's a real reliability
  tradeoff, not free money.
- **Aggressive right-sizing removes headroom**, which is exactly what
  absorbs bursts (topic 07, A11). Right-sizing to p50 utilisation guarantees
  latency problems at peak. Size to peak plus margin.
- **Single-AZ to avoid cross-AZ charges** is almost always wrong — you're
  trading an availability property for a line item. The correct fix is
  topology-aware routing that *prefers* local and *fails over* cross-AZ.
- **Reducing replica counts or replication factor** directly reduces
  durability and availability. `min.insync.replicas=1` to save a broker is
  the Kafka version (topic 07, A10).
- **Shortening retention** reduces your ability to investigate incidents and
  may breach compliance.
- **Over-committing** to Savings Plans locks you into an architecture, which
  is a real cost if you're planning a migration (A14).

**The organisational lever that matters most**: **cost attribution**.
Mandatory tagging and per-team showback. Engineers optimise what they can
see; a central FinOps team chasing 200 engineers' spend cannot win, and a
team that sees its own monthly number usually fixes the obvious things
without being asked.

**Weak answers miss.** Commitments as the first and largest lever, and being
explicit about which savings cost reliability. An answer that lists
optimisations without naming the tradeoffs is a cost answer, not an
engineering answer.

**Follow-ups to expect.**
- How do you attribute shared costs (the cluster, the NAT gateway, the
  monitoring stack)? (Split by a usage proxy — pod resource requests for
  cluster cost, flow logs for NAT. Imperfect but directionally right, and
  visible-and-approximate beats invisible-and-exact.)

---

### A12. Managed vs self-hosted, applied to Kafka

**Answer.**
**The framework** — five questions, in order:

1. **Is this a core differentiator?** If running it better than the market
   is part of your product's advantage, self-host. If it's undifferentiated
   plumbing, buy. Most infrastructure is plumbing.
2. **What is the fully-loaded cost of self-hosting?** Not the instance bill:
   engineer time to build, operate, upgrade, and be on-call for it —
   typically a meaningful fraction of one or more FTEs, forever — plus the
   opportunity cost of what those people aren't doing. Compare that against
   the managed premium honestly. Managed services often look expensive until
   you price 0.5 FTE.
3. **Do we have, and can we retain, the expertise?** A self-hosted system
   with one expert is a single point of failure that will resign. Three
   people who can debug it at 3am is the real bar.
4. **Does the managed service actually meet the requirement?** Version
   support, configuration access, performance ceilings, compliance and data
   residency, network topology (does it support PrivateLink?), and the
   specific features you rely on. Managed services constrain you in ways
   that are invisible until they bite.
5. **What is the exit cost?** If this becomes wrong in two years, how hard
   is it to leave? Standard protocols and open formats are worth paying for;
   proprietary APIs raise the exit cost sharply.

**Applied to Kafka:**

*Arguments for managed (MSK, Confluent Cloud):*
- Kafka's operational burden is genuinely high and genuinely specialised:
  partition rebalancing, broker replacement without data loss, upgrade
  sequencing, ZooKeeper→KRaft migration, disk sizing and retention tuning,
  ISR and quota management. Getting these wrong loses data (topic 07, A10).
- The failure modes are subtle and the recovery procedures require practice
  you only get from incidents.
- Confluent Cloud in particular removes capacity planning almost entirely,
  and the ecosystem (Schema Registry, Connect, ksqlDB) is operated for you —
  and those components are individually as much work as Kafka itself.
- For most companies, Kafka is undifferentiated plumbing.

*Arguments for self-hosting:*
- **Cost at high volume.** This is the strongest argument and it's
  quantitative. Kafka at hundreds of MB/s sustained is a large managed bill;
  self-hosted on right-sized instances with local NVMe — or bare metal, where
  you also avoid cross-AZ transfer charges entirely (topic 04, A13) — can be
  several times cheaper. At 500 MB/s+ the delta funds the team.
- **Configuration control**: broker-level tuning, custom retention per
  topic, unusual replication topologies, specific versions, and interceptors
  or plugins that managed offerings don't permit.
- **Data locality**: if Kafka must sit next to a bare-metal ClickHouse
  cluster on local NVMe, a managed cloud service is in the wrong place and
  the network cost and latency reflect it.
- **Compliance** requiring you to control the storage and the keys.
- You already have deep expertise and are already operating it.

*My decision:* **volume is the deciding variable.** Below roughly tens of
MB/s sustained, managed — the cost delta doesn't justify a specialist, and
you should spend the headcount elsewhere. Above a few hundred MB/s with a
team that already runs it, self-hosted, because the economics invert and you
likely also need the control. In between, decide on expertise availability
and on whether you have adjacent workloads (ClickHouse, Elasticsearch) that
already justify the storage-operations skillset — which is exactly the case
where self-hosting amortises across several systems.

I'd also say: **the decision isn't permanent**, and designing the
application to be agnostic (standard client protocol, no proprietary
extensions) keeps the exit cheap in either direction, which is worth a small
amount of upfront discipline.

**Weak answers miss.** Fully-loaded cost including on-call and the
bus-factor question, and volume as the quantitative crossover.

**Follow-ups to expect.**
- MSK vs Confluent Cloud? (MSK is managed brokers — you still own topic
  design, client configuration, partition strategy, and much of the tuning.
  Confluent Cloud abstracts further and includes the ecosystem, at a higher
  price and more lock-in. "Managed" is a spectrum, not a binary, and
  knowing where a service sits on it is the real skill.)

---

### A13. What actually fails in a "region outage"

**Answer.**
First correction: **regions rarely fail completely.** What actually happens
is that one service, or one AZ, or the *control plane* degrades — and the
resulting failures propagate in ways that surprise people who tested only
for "an AZ went away."

**The surprises, in rough order of how often they catch teams out:**

1. **Control plane vs data plane.** Your running instances keep running and
   your load balancers keep balancing, but you **cannot make changes**: no
   launching instances, no scaling, no modifying security groups, no
   updating DNS records, no assuming roles if STS is affected. So your
   *automated recovery* is exactly what stops working. Autoscaling can't add
   capacity; your failover automation can't provision the DR environment;
   your deploy pipeline can't ship a fix. Every DR plan that depends on
   creating resources in the moment assumes a working control plane, and
   that's the assumption a real event violates. **Design so that failover is
   traffic shifting to already-provisioned capacity, not provisioning.**
2. **Global services are hosted somewhere.** Several AWS "global" services
   have a home region (historically us-east-1) for their control plane —
   IAM writes, Route 53 configuration changes, CloudFront distribution
   updates, some billing and Organizations operations. So a us-east-1 event
   can prevent DNS *changes* worldwide, even though DNS *resolution*
   continues. Verify the current architecture rather than assuming, but the
   principle holds: know which of your dependencies have a single-region
   control plane.
3. **Everyone fails over at once.** Your DR region now receives your traffic
   *and* every other customer's. **Capacity is not guaranteed** —
   `InsufficientInstanceCapacity` for your instance type is a documented
   reality during large events. Pilot-light designs that assume they can
   scale from zero are the most exposed. Mitigation: reserved capacity or
   warm standby, and instance-type diversity.
4. **Quotas in the DR region.** You've never run production there, so your
   limits are defaults: EC2 vCPU quotas, ENI limits, Lambda concurrency, ELB
   counts. And **raising a quota requires the control plane and a support
   response** at the worst possible time. This is a boring, extremely common
   failure — audit and pre-raise DR quotas as routine work.
5. **Cross-region dependencies you forgot.** An S3 bucket in the failed
   region; a Secrets Manager secret; an ECR repository your DR nodes pull
   images from; an artifact store; a Terraform state backend; a CI system;
   your **monitoring and paging** infrastructure. A DR region whose nodes
   pull images from the dead region cannot start.
6. **Data.** Async replication means RPO > 0 (A5), and promoting a replica
   is a one-way door if you don't have reverse replication. Multi-region
   stores with LWW may have silently lost writes (topic 07, A15).
7. **DNS TTLs and client caching** mean the traffic shift is not clean
   (topic 02, A12) — a long tail of clients keeps hitting the dead region.
8. **Third parties** — your payment processor, your identity provider, your
   observability vendor — may themselves be single-region, or be in the same
   region, or be degraded because everyone is failing over.
9. **Human factors**: the runbook is in a wiki hosted in the affected
   region; the on-call can't log in because SSO depends on something
   affected; nobody has practised the procedure.

**What a multi-AZ deployment actually protects you from**: a single AZ's
power, cooling, or network failure. That's real and it's the most common
infrastructure failure, so multi-AZ is genuinely valuable. It does **not**
protect you from: a regional service's control plane degrading, a bad
configuration deployed everywhere, a software bug, a dependency's outage, or
your own bad deploy — which together are the cause of most outages.

**What I'd actually do about it:**
- Design failover as **traffic shifting to running capacity**, never as
  provisioning.
- **Pre-provision and pre-quota** everything in the DR region, and run
  *something* real there continuously so it isn't cold.
- **Inventory single-region dependencies** explicitly, including your
  tooling, and treat that inventory as a maintained artifact.
- **Drill it.** Quarterly, with real traffic, measuring the achieved RTO.
  Everything above is discovered in the first drill and in no other way.

**Weak answers miss.** Control-plane-vs-data-plane, which is the single most
important concept in this answer, and the DR-region quota problem.

**Follow-ups to expect.**
- Multi-region or multi-cloud for resilience? (Multi-region handles almost
  everything and is far cheaper. Multi-cloud is justified by provider-wide
  failure or by commercial/regulatory requirements — and it costs you the
  lowest common denominator of both providers' capabilities plus double the
  operational expertise. Usually not worth it for resilience alone; say so
  plainly.)

---

## Tier 3 — Scenario / debug

### A14. AWS → GCP over 18 months

**Answer.**
Open with the questions that determine everything, because a candidate who
starts drawing architecture has skipped the important part:
**Why?** Cost, a commercial agreement, a capability (BigQuery, GPUs,
Spanner), or a mandate? The answer changes the sequencing and the
success criteria entirely — a cost-driven migration means measuring cost
per workload continuously; a capability-driven one means you may only move
part of the estate. And **is the goal to leave AWS entirely, or to become
multi-cloud?** Those are different programmes; "leave" has an end date and a
decommissioning plan, "multi-cloud" is a permanent doubling of operational
surface that needs to be a deliberate choice.

I'd also state the biggest risk up front: **the interim period is the
dangerous part, not the endpoint.** For most of 18 months you're running
both, paying for both, with services split across a WAN. Every design
decision should minimise the duration and the cross-cloud chatter of that
interim state.

**Phase 0 — Discovery and foundations (months 0–3).**

- **Inventory.** Every service, its dependencies (both directions), its
  data stores and their sizes, its traffic volumes, its SLOs, its
  compliance constraints, and its owning team. Automated discovery
  (VPC flow logs, service mesh telemetry, CloudTrail, Config) plus team
  interviews — because the dependency you don't know about is the one that
  breaks the cutover.
- **Build the dependency graph and identify the cut points.** The graph
  determines the order: you move leaves first, and you look for weakly
  connected clusters that can move together. A service whose dependencies
  span the cut will be chatty across the WAN.
- **Classify each workload** by strategy: rehost (lift and shift), replatform
  (EKS→GKE, RDS→Cloud SQL), refactor (rewrite for a native service),
  repurchase, retain (stays on AWS), retire (turn it off — and there is
  always more of this than expected; a migration is an excellent excuse to
  delete things).
- **Foundations in GCP**: organisation, folder/project structure mirroring
  your AWS account strategy (A10), IAM and SSO federation, VPC design with
  **non-overlapping CIDRs against your AWS ranges** (topic 04, A10 — get
  this right on day one or you will build a NAT layer you can never remove),
  logging, monitoring, and the IaC pipeline.
- **Connectivity**: Cloud Interconnect or a Partner Interconnect between AWS
  and GCP, redundant, with a VPN backup, BGP-routed (topic 04, A8/A16).
  Sized for the interim cross-cloud traffic *plus* the bulk data transfers.
  This is a long-lead-time item — start it in month 0.
- **Identity**: federate both clouds to the same IdP so engineers have one
  login, and establish workload identity federation so services can
  authenticate across the boundary without static keys.
- **Pick and build the pilot**: a low-risk, non-critical,
  representative service. Its purpose is to exercise the whole pipeline —
  IaC, CI/CD, networking, observability, on-call — and to discover the
  unknowns cheaply.

**Phase 1 — Platform and pilot (months 3–6).**

- Make CI/CD **target-agnostic**: the same pipeline deploys to EKS or GKE
  based on configuration. If teams need different pipelines per cloud, the
  migration will stall.
- **Observability spanning both**, in one place, from the start. You cannot
  operate a split estate with two monitoring stacks and two on-call
  experiences. This is the piece that's most often deferred and most often
  regretted.
- Migrate the pilot end to end, including a **rollback**. Measure everything:
  effort, surprises, latency changes, cost delta.
- Write the **migration runbook template** from what the pilot taught you.
  Every subsequent service follows it.

**Phase 2 — Bulk migration (months 6–15).**

Sequencing principles:
- **Move data and its consumers together.** Data gravity dominates. A
  service in GCP querying a database in AWS pays 10–50 ms per round trip; a
  request making 20 sequential queries becomes a timeout. **This is the
  single most common way cross-cloud migrations fail**, and it means the
  unit of migration is usually a *service + its data*, not a service alone.
- **Leaves first**, then inward. Or move a weakly-connected cluster of
  services as one unit.
- **Stateless before stateful.**
- Batch and analytics workloads early — they tolerate latency and prove the
  data pipeline.
- Anything with an unavoidable cross-cloud call: keep it on AWS until its
  dependency moves.

Per-service pattern, which is the same shape every time:
1. Deploy to GCP alongside AWS. Both running.
2. Replicate data continuously (topic 06, A16 for the database mechanics).
3. Shift a **small percentage of traffic** and compare error rate, latency,
   and cost. Global load balancing or weighted DNS as the dial.
4. Ramp to 100%.
5. Bake, then decommission the AWS side.
6. **Rollback is shifting traffic back** — which requires reverse data
   replication to have been established first. Non-negotiable.

For the databases specifically, the full programme is topic 06, A16 — and I
would explicitly reuse that answer rather than reinvent it: inventory
extensions and collations, logical replication, verification by checksum and
shadow reads, sequences, a scripted cutover with a short write pause, and
reverse replication for rollback.

**Phase 3 — Decommission and prove (months 15–18).**

- Turn things off deliberately, with a stop-then-delete gap so mistakes are
  recoverable.
- **Cancel AWS commitments** — Savings Plans and RIs are a real cost of the
  migration and must be modelled in the business case from the start (A11);
  a 3-year commitment purchased in month 2 is money burned.
- Verify the cost outcome against the original justification, honestly.
- Post-migration review.

**How to avoid a big bang** — the explicit answer:
- **Strangler fig**: route through a facade (a global load balancer, an API
  gateway, or a service mesh spanning both) so individual routes can move
  independently and invisibly to callers. Nothing is ever "switched"; things
  are shifted, one at a time, reversibly.
- **Every step reversible**, with reverse replication where data is involved.
- **Traffic percentage as the migration dial**, so exposure is continuous
  rather than binary.
- **Never move two things at once** — either the compute or the data, not
  both in one step, so that when something breaks you know which change
  caused it.

**Risks I'd raise unprompted:**
1. **Cross-cloud latency and egress during the interim.** Both a
   performance and a cost risk, and both scale with how long the interim
   lasts — which is the argument for aggressive sequencing.
2. **Paying for both clouds** for most of the programme. The business case
   must include it; migrations that were sold on cost savings often go over
   because nobody modelled the overlap.
3. **Team capacity and fatigue.** 18 months of migration alongside feature
   work is the most common reason these stall at 70% — and a stalled
   migration is the worst outcome, because you permanently own two platforms.
   I'd push for a dedicated migration team plus embedded owners, and for an
   explicit decision that some things will be *retired* rather than moved.
4. **Managed-service gaps**: not everything has a clean equivalent (DynamoDB,
   Kinesis, Step Functions, specific RDS extensions), and each gap is a
   refactor with its own risk.
5. **Skills**: the team's AWS depth doesn't transfer for free. Budget
   training time, and expect the first months to be slower than planned.

**Weak answers miss.** Data gravity as the sequencing driver, the interim
period being the risky part, and the commitment/overlap cost. A candidate
who produces a service-by-service plan without addressing cross-cloud
latency has designed something that will fail at the third service.

**Follow-ups to expect.**
- What would make you recommend *not* doing it? (If the driver is cost and
  the modelled saving is under ~30%, the migration's cost and risk likely
  exceed the benefit — say the number. If the driver is a single capability,
  a targeted multi-cloud deployment of that one workload is far cheaper than
  moving everything.)
- How do you keep the team motivated? (Visible progress metrics, migrate
  something valuable early, and — genuinely — use the migration to delete
  things and fix long-standing pain, so it isn't purely a lateral move.)

---

### A15. First 90 days after an acquisition

**Answer.**
The organising principle: **integrate in the order of risk, not in the order
of tidiness.** Nothing about consolidating tooling matters if a production
incident on their side has no path to your on-call, or if a credential
nobody owns is still valid.

**Days 0–14 — Contain risk and see what you own.**

1. **Security first, and immediately.**
   - Audit IAM: who has access, which credentials are long-lived, which
     belong to departed employees or to the previous parent company.
   - Rotate anything shared or unattributable. Disable unused users.
   - Enable CloudTrail/Config/GuardDuty if absent, with logs going somewhere
     they can't be deleted from.
   - Check for public S3 buckets, open security groups, unencrypted volumes,
     and internet-exposed management endpoints. This is the highest-value
     two weeks of work in the whole programme, and it's often the first time
     anyone has looked.
2. **Get visibility.** Their monitoring into your dashboards, or at minimum
   their alerts into your paging system. **You cannot own something you
   cannot see**, and an acquired system with a separate observability stack
   is a system you don't operate.
3. **Establish joint on-call and escalation.** Not merged rotations yet —
   just a documented path so an incident on either side reaches the right
   people. Their engineers stay on their systems; you add a liaison.
4. **Inventory**: accounts, clusters, services, data stores, domains,
   third-party contracts, and — critically — **who knows how each thing
   works**. Key-person risk is the biggest hidden liability in an
   acquisition, and retention conversations are urgent, not HR's problem for
   later.
5. **Cost snapshot**: their bill, its trajectory, and any commitments that
   constrain future decisions.

**Days 14–45 — Connect, without merging.**

6. **Networking.** The 20+ services must talk to yours. Address the CIDR
   question immediately (topic 04, A10):
   - Check for overlap. If their VPC and yours are both `10.0.0.0/16`, you
     cannot peer or route, and you have three options: re-address (correct,
     slow), NAT bridge (pragmatic, painful for DNS and observability), or
     avoid routing entirely.
   - **My default: expose the specific integrations via PrivateLink** (or
     public endpoints with mTLS) rather than joining the networks. It works
     regardless of CIDR overlap, it's unidirectional, it's granular, and it
     doesn't create a shared blast radius. Full network merge is a project;
     three PrivateLink endpoints is a week.
   - If broader connectivity is genuinely needed, Transit Gateway with
     cross-account attachment (and re-addressing one side on a funded
     timeline).
   - Register their CIDRs in your IPAM whatever you do, or you'll repeat
     this with the next acquisition.
7. **Identity federation** — their engineers into your SSO, and cross-account
   roles for your platform team, so access is auditable and revocable
   centrally.
8. **DNS**: decide the naming strategy and set up resolution both ways
   (topic 04, A12).

**Days 45–90 — Rationalise deliberately.**

9. **Decide what merges and what doesn't**, explicitly and in writing.
   Categories:
   - **Merge now**: identity, security tooling, observability, on-call,
     incident process, cost reporting. These are cross-cutting and the
     benefit is immediate.
   - **Merge later**: CI/CD, IaC (their CloudFormation/CDK into your
     Terraform, or vice versa), Kubernetes clusters, container registries.
     High effort, moderate benefit, and safe to defer.
   - **Never merge**: things about to be retired. Identify these early —
     an acquisition usually means duplicate products, and migrating
     something you'll turn off in a year is pure waste.
10. **Redundancy elimination** — duplicate monitoring vendors, duplicate
    log platforms, duplicate CI, duplicate cloud accounts sitting idle.
    Usually the fastest cost win, and it also reduces the surface you must
    operate.
11. **Standardise the IaC.** This is where the real work is: their stack is
    likely partially manual, with drift. Bringing it under your tooling means
    importing, reconciling, and rewriting — and the goal is *no unmanaged
    resources*, which matters more than which tool wins (A6).
12. **Plan the service integration properly.** For each of the 20+ services:
    does it call yours, do yours call it, what's the latency budget, what's
    the auth mechanism? Integration by shared Kafka topics or an API gateway
    with mTLS is far more robust than assuming network reachability.

**What I'd explicitly not do in 90 days:**
- Merge the Kubernetes clusters. High risk, low immediate value, and it
  forces every workload to change at once.
- Force their team onto your tooling wholesale. It destroys velocity and
  goodwill at exactly the moment you need both, and you'll lose the people
  who know how their systems work.
- Re-address their entire network unless there's no alternative — fund it
  as a project with a timeline, don't attempt it in the first quarter.

**The thing I'd flag as the real risk:** not technology. It's that the
acquired team's knowledge is undocumented and their motivation is uncertain.
Every technical decision above should be weighed against whether it keeps
those engineers engaged, because a migrated system whose experts have left
is a system nobody can operate. Concretely: involve them in the decisions,
give them ownership of the integration work rather than doing it to them,
and document as you go.

**Weak answers miss.** Security rotation in week one, PrivateLink as the
low-friction integration path, and the key-person risk. Also missed: being
explicit about what *not* to merge, which is what makes a 90-day plan
achievable rather than aspirational.

**Follow-ups to expect.**
- How do you handle two different Kubernetes platforms long-term? (Either
  converge on one over a year with a shared platform API so teams don't
  care, or run both with a common control plane (GitOps, mesh) and converge
  opportunistically. Pick based on how different they are and whether either
  is genuinely better.)

---

### A16. Bill up 40%, traffic flat

**Answer.**
"Traffic flat" is the useful clue: the growth is **not** from serving more
users, so it's either something new, something that stopped being cleaned
up, something that changed shape, or a pricing/commitment change.

**Step 1 — Get the shape of the increase before theorising.**
Cost and Usage Report (or the BigQuery billing export), grouped by
**service**, **usage type**, **region**, and **account/project**, comparing
the two quarters. Then diff. In most cases one or two line items explain
most of the delta, and this takes an hour. Anyone who starts by inspecting
instances is searching linearly.

Then ask **when** it grew: a step change points at a specific event (a
deploy, a new feature, a commitment expiry); a steady ramp points at
accumulation (data growth, resource leakage).

**Step 2 — Work through the candidates the shape suggests.**

*If it's compute:*
- **Commitment expiry.** Savings Plans or RIs expired and the same usage
  reverted to on-demand — a 30–60% jump with zero infrastructure change.
  Check coverage and utilisation reports **first**; it's the single most
  common cause of a step change and the easiest to fix.
- **Autoscaling floor raised**, or a min-replicas change, or a request/limit
  increase that reduced pod density and forced more nodes.
- **Instance family change** in a Terraform module default or an AMI update.
- **A new environment** — a staging or load-test cluster left running.
- **Karpenter/ASG churn** or a failure to consolidate.

*If it's data transfer:*
- **Cross-AZ** growth from a topology change, a mesh rollout, or a
  replication factor increase (topic 04, A13). Invisible in traffic
  metrics because it's internal.
- **NAT gateway processing** — a new workload pulling large images or data
  through NAT instead of a VPC endpoint (A11 / topic 04, A3). A frequent and
  easily fixed finding.
- **Cross-region** — a new replica, a backup target, or a multi-region
  feature.
- Egress from a new integration or a partner pulling data.

*If it's storage:*
- **No lifecycle policy** on a growing bucket, or S3 **versioning** with no
  expiration on noncurrent versions (this one compounds silently and is
  frequently the answer to a steady ramp).
- **Incomplete multipart uploads** accumulating — invisible in the console's
  object count, billable, and cleared by a lifecycle rule.
- Snapshot proliferation; unattached EBS volumes from terminated instances.
- **Log or metric retention** increased, or a service that started logging
  far more (topic 09, A12) — and note this shows up as storage *and* as
  ingest/indexing compute.

*If it's managed services:*
- An observability vendor's bill scaling with **cardinality**, not traffic
  (topic 09, A5) — a new label or a new service can multiply series count
  with no traffic change at all. This is a very common answer for
  monitoring-heavy companies and it fits "flat traffic" perfectly.
- A database scaled up, or storage auto-growing and never shrinking (RDS
  storage doesn't shrink).
- KMS request charges from a new encryption pattern without data key
  caching.

**Step 3 — Attribute it.** Break the delta down by tag/account/team. If
tagging is poor, that's a finding in itself and the first remediation, since
you can't hold anyone accountable for a number nobody owns.

**Step 4 — Fix, in order of effort:** re-commit expired Savings Plans (hours,
largest impact); delete waste — unattached volumes, old snapshots, idle
environments (days); add lifecycle policies and VPC endpoints (days);
right-size (weeks, continuous); architectural change (months).

**Step 5 — Prevent recurrence**, which is the part that makes this a staff
answer rather than an investigation:
- **Anomaly detection** on cost, per service and per account, alerting on a
  step change — so the next one is caught in days, not in a quarterly
  review.
- **Budgets with alerts** per team.
- **Cost visibility in the deploy path** — Infracost or equivalent on
  Terraform PRs, so the cost of a change is visible before it merges.
- **Showback**: each team sees its own spend monthly. This does more than
  any central optimisation effort.
- **Automated waste detection** as a scheduled job, not a periodic project.
- A **commitment review calendar** so expiries never surprise you again.

**Weak answers miss.** Checking commitment expiry first, and cardinality-
driven observability cost — both fit "flat traffic, higher bill" precisely,
and both are missed by people who go straight to instance right-sizing.

**Follow-ups to expect.**
- What if it's spread evenly across everything with no single cause?
  (Then it's likely a pricing/commitment change, a currency or region
  change, or genuine broad growth in a dimension you're not measuring — data
  volume rather than request volume, for instance. Storage and data grow
  even when traffic doesn't.)

---

### A17. Sizing a fleet you've never run

**Answer.**
The honest framing: **you will be wrong.** The goal isn't an accurate
estimate, it's an estimate with a known error bar and a cheap path to
correct it. So the method is: model, measure, then make the system adaptive.

**Step 1 — Model from first principles, and write down the assumptions.**
- Derive expected load: users × actions per user per day, converted to
  average and **peak** QPS. Peak-to-average for a consumer service is
  commonly 2–5x; for a business-hours service with one time zone it can be
  much higher. State the ratio you're assuming.
- Estimate per-request resource cost. If you have nothing to go on, bound
  it: what's the dominant operation, how many bytes does it touch, how many
  downstream calls does it make?
- Apply **Little's law** (topic 07, A11): `concurrency = throughput ×
  latency`. If you expect 5000 req/s at 50 ms, you need 250 concurrent
  request slots. That converts directly into threads, connections, and
  instance count.
- Target **50–70% utilisation at peak**, not 90 — the queueing curve means
  90% is the latency cliff. This single decision usually dominates the
  answer.
- Do storage and bandwidth separately: bytes per event × events per second
  → GB/day → retention → total, with compression assumptions stated.

**Step 2 — Load test the real thing, because the model is a hypothesis.**
- Test a **single instance to saturation** first. That gives you the unit
  capacity, which is the number everything else is derived from, and it's
  far more informative than testing the whole fleet.
- Find the **knee**: the load at which latency starts climbing
  super-linearly. Your operating point is below that, and the distance below
  it is your burst headroom.
- Test with **realistic data and traffic shape**: real payload sizes, real
  cache hit rates, real key distribution. A load test with a uniform key
  distribution against a cache will report a hit rate you'll never see, and
  will tell you the system is several times faster than it is. This is the
  most common way load tests lie.
- Test the **failure and recovery** path, not just the peak: what happens
  above the knee, and does it recover when load drops (topic 07, A16)?
- Identify which resource saturates first — CPU, memory, network, disk, a
  connection pool, a lock. That determines what to scale and what to fix.

**Step 3 — Account for what the model omits.**
- **Failure domains**: to survive losing one AZ of three, you need 1.5x the
  peak capacity, not 1x. To survive an instance failure at peak, N+1 at
  minimum.
- **Deployment headroom**: a rolling deploy with `maxUnavailable: 25%`
  removes a quarter of your capacity while it runs.
- **Downstream limits**: your fleet size multiplies database connections
  (topic 06, A8), and the data tier may bind before the app tier does. Size
  the whole path, not one layer.
- **Cold start / warm-up** cost when scaling (topic 03, A16).
- **Growth**: size for a defined horizon (say 6 months) so you're not
  re-sizing monthly, but no further — over-provisioning for a 3-year
  projection is expensive and probably wrong.

**Step 4 — Make being wrong cheap**, which is the actual answer to the
"expensive direction" part of the question:
- **Autoscale**, on a leading signal (concurrency or queue depth, not CPU —
  topic 08, A10), so the fleet finds its own size. This turns a sizing
  estimate into a starting point rather than a commitment.
- **Start smaller than the estimate and ramp**, with headroom monitoring —
  it's much cheaper to add capacity in week two than to run 3x for a year.
  The asymmetry matters: under-provisioning is visible immediately and
  fixable in minutes; over-provisioning is invisible and persists.
- **Don't buy commitments until usage is stable** (A11) — commit to the
  observed baseline after a couple of months, not to the projection. This is
  the specific mechanism by which over-sizing becomes expensive and
  irreversible.
- **Instrument headroom explicitly**: a dashboard showing current load
  against measured saturation point, per component. That's the number that
  tells you when to act, and almost nobody has it.
- **Book a re-sizing review** at 30 and 90 days, with real data.

**What I'd say to the interviewer explicitly:** the estimate is for
procurement and architecture decisions; the load test is what I'd actually
trust; and the autoscaling plus headroom monitoring is what makes it safe to
be wrong. If someone demands a precise number without a load test, the
honest answer is a range with the assumptions attached, not a false
precision.

**Weak answers miss.** Load-testing one instance to saturation as the
primary method, the realistic-data caveat, and delaying commitments — which
is the concrete way you avoid being wrong expensively.

**Follow-ups to expect.**
- What if you can't load test (a third-party dependency, no test
  environment)? (Shadow traffic against the real dependency where possible;
  otherwise start conservatively, ramp real traffic in percentage steps with
  a fast rollback, and treat the ramp itself as the load test. Say that this
  is riskier and requires better monitoring, rather than pretending it's
  equivalent.)

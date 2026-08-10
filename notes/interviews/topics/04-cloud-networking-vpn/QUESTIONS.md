# Cloud Networking, VPN & IPsec — Questions

Hybrid and multi-cloud connectivity, which is where "we acquired a company"
and "the customer needs a private link" turn into architecture. The IPsec
questions here get asked verbatim at infrastructure-heavy companies; the CIDR
overlap question is the one that actually shows up in your working life.

16 questions.

---

## Tier 1 — Recall

### Q1. What makes a VPC subnet "public" or "private"? Walk the route table.
*Tags: vpc, routing*

### Q2. Compare security groups and network ACLs. Why does the stateful/stateless difference matter in practice?
*Tags: vpc, firewall, aws*

### Q3. What does a NAT gateway actually do, and name two ways it becomes a problem at scale.
*Tags: nat, cost, ports*

### Q4. In a site-to-site IPsec VPN, what are IKE phase 1 and phase 2 responsible for? What is an SA?
*Tags: ipsec, vpn, ike* · *[asked verbatim]*

### Q5. What is the difference between ESP and AH, and between tunnel mode and transport mode?
*Tags: ipsec, vpn*

### Q6. What is an autonomous system, and what is the difference between eBGP and iBGP?
*Tags: bgp, routing*

---

## Tier 2 — Explain / compare

### Q7. Why do IPsec tunnels break large packets, and what is the standard fix?
*Tags: ipsec, mtu, debugging* · *[infra-heavy]*

### Q8. Compare a site-to-site VPN, a dedicated interconnect (Direct Connect / Cloud Interconnect), VPC peering, and a Transit Gateway. When is each the right answer?
*Tags: hybrid, connectivity, aws*

### Q9. Compare WireGuard and IPsec for a site-to-site tunnel. What does each get right?
*Tags: vpn, wireguard, ipsec*

### Q10. You acquire a company. Their VPC is `10.0.0.0/16`. So is yours. You need the two environments to talk. What are your options?
*Tags: cidr, nat, migration* · *[real-world]*

### Q11. Compare VPC peering, Transit Gateway, and PrivateLink. Why is peering non-transitive, and what does PrivateLink solve that peering can't?
*Tags: vpc, connectivity, aws*

### Q12. Explain how DNS resolution works for a hybrid environment where some names are in a private cloud zone and some are on-premises.
*Tags: dns, hybrid, split-horizon*

### Q13. Where does cloud network traffic cost money? Build a mental model an engineer can use when designing.
*Tags: cost, egress, architecture*

---

## Tier 3 — Scenario / debug

### Q14. An EC2 instance in a private subnet can't reach an external API. Give me your checklist, in order, and say what each step eliminates.
*Tags: debugging, vpc, method*

### Q15. Your site-to-site VPN to a partner's datacentre works, but throughput tops out around 1 Gbps no matter what you do, and it drops entirely for a few seconds roughly every hour. Diagnose both symptoms.
*Tags: ipsec, throughput, rekey, debugging* · *[infra-heavy]*

### Q16. Design connectivity for a company running production in AWS across three regions, a bare-metal colo for GPU workloads, and a requirement that a customer can reach your API over a private link without traversing the internet. Draw the layers.
*Tags: design, hybrid, privatelink*

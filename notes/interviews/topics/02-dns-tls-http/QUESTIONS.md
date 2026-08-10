# DNS, TLS & HTTP — Questions

The layer where most production incidents actually live. DNS questions test
whether you understand caching you don't control; TLS questions test whether
you've operated certificates rather than clicked "enable HTTPS"; HTTP
questions test whether you know what a retry costs.

17 questions.

---

## Tier 1 — Recall

### Q1. Walk through what happens when a resolver looks up `api.example.com` and nothing is cached anywhere.
*Tags: dns, resolution*

### Q2. Why can't a CNAME exist at a zone apex? What do providers offer instead?
*Tags: dns, records*

### Q3. What is negative caching in DNS, and which value controls it?
*Tags: dns, caching, ttl*

### Q4. What does certificate chain validation actually check, step by step?
*Tags: tls, pki*

### Q5. What is SNI, why was it needed, and what does it leak?
*Tags: tls, privacy*

### Q6. Which HTTP methods are safe, which are idempotent, and why does the distinction matter operationally?
*Tags: http, retries, semantics*

---

## Tier 2 — Explain / compare

### Q7. Compare the TLS 1.2 and TLS 1.3 handshakes. How many round trips each, and what did 1.3 remove?
*Tags: tls, latency*

### Q8. What is 0-RTT in TLS 1.3, and what is the risk you take on by enabling it?
*Tags: tls, latency, security*

### Q9. Explain the options for certificate revocation. Why is revocation considered mostly broken, and what is the industry doing instead?
*Tags: tls, pki, ocsp*

### Q10. What does mTLS give you that TLS doesn't, and what are the operational costs at fleet scale?
*Tags: tls, mtls, zero-trust* · *[infra-heavy]*

### Q11. Compare GeoDNS and anycast for steering users to a nearby point of presence. What can each do that the other can't?
*Tags: dns, anycast, traffic-management*

### Q12. Why is DNS a poor failover mechanism? If you have to use it, how do you make it as good as possible?
*Tags: dns, failover, ttl*

### Q13. Explain `Cache-Control: no-cache` vs `no-store` vs `private` vs `must-revalidate`, and how `ETag` fits in.
*Tags: http, caching*

### Q14. Walk through a CORS preflight. What triggers one, and what does the browser do with the response?
*Tags: http, cors, browser*

---

## Tier 3 — Scenario / debug

### Q15. You deploy a change and a fraction of clients start failing TLS handshakes — but only clients from one country, and only some of them. Your certificate is valid. What's your investigation?
*Tags: tls, debugging, pki*

### Q16. Your API returns 500s for ~2% of requests during a partial outage. Your client library retries three times with no delay. Describe what happens to the system, and design the retry policy you'd actually ship.
*Tags: http, retries, resilience* · *[infra-heavy]*

### Q17. A team wants to cut a 200 ms p50 on a mobile API. Walk through everything in the DNS/TLS/HTTP path you'd examine, with rough budgets for each.
*Tags: latency, tls, http, dns*

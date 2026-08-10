# TCP/IP & the Wire — Questions

The most reliably asked infrastructure topic, and the one where senior
candidates most often turn out to be shallow. Interviewers use it as a
proxy for "has this person actually debugged a network problem, or only
read about one." Expect Tier 1 recall at product companies and Tier 3
debugging at infrastructure companies.

18 questions.

---

## Tier 1 — Recall

### Q1. Walk me through the TCP three-way handshake. What is in each segment, and why three messages rather than two?
*Tags: tcp, handshake*

### Q2. What is the difference between MTU and MSS? How does each get chosen?
*Tags: tcp, ip, mtu*

### Q3. What is `TIME_WAIT`, which side enters it, and how long does it last?
*Tags: tcp, teardown, state-machine*

### Q4. What does a socket in `CLOSE_WAIT` mean, and whose bug is it?
*Tags: tcp, teardown, debugging*

### Q5. What is the difference between a TCP `RST` and a `FIN`? Name three situations that produce a `RST`.
*Tags: tcp, teardown*

### Q6. Explain the difference between TCP flow control and TCP congestion control.
*Tags: tcp, congestion*

### Q7. What is the bandwidth-delay product, and what is it used for?
*Tags: tcp, throughput, wan*

---

## Tier 2 — Explain / compare

### Q8. Compare TCP CUBIC and BBR. What problem was BBR built to solve, and where does it behave worse?
*Tags: tcp, congestion, wan* · *[infra-heavy]*

### Q9. Explain Path MTU Discovery. Describe the failure mode where it silently breaks, and how you'd detect it.
*Tags: ip, mtu, icmp, debugging* · *[infra-heavy]*

### Q10. What is the interaction between Nagle's algorithm and delayed ACK, and what does it look like to an application?
*Tags: tcp, latency*

### Q11. When would you choose UDP over TCP for a service you own? What do you have to rebuild yourself?
*Tags: udp, protocol-design*

### Q12. Explain head-of-line blocking. How does it differ between HTTP/1.1, HTTP/2, and HTTP/3?
*Tags: tcp, quic, http*

### Q13. Compare TCP keepalive with an application-level heartbeat. Which one detects what, and how fast?
*Tags: tcp, failure-detection*

### Q14. What is a SYN flood, and how do SYN cookies defend against it? What do you give up by enabling them?
*Tags: tcp, security, ddos*

### Q15. Why is IP fragmentation considered harmful, and what changed in IPv6?
*Tags: ip, mtu, ipv6*

---

## Tier 3 — Scenario / debug

### Q16. A service behind your load balancer starts intermittently stalling for almost exactly 5 seconds on a small fraction of requests. Nothing in the application logs looks slow. Where do you look, and in what order?
*Tags: debugging, dns, tcp, latency* · *[infra-heavy]*

### Q17. You run an egress proxy that opens connections to a small number of upstream hosts on behalf of thousands of clients. Under load it starts failing to make new connections, and the error is not a timeout. What's happening, what's the arithmetic, and what are your options?
*Tags: tcp, ports, nat, capacity*

### Q18. You suspect packet loss between two of your datacentres. You have root on both ends. Walk me through what you actually run, and how you distinguish loss from reordering, from a slow application, from a congested link.
*Tags: debugging, tooling, tcp*

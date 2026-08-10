# The Take-Home + Live Extension Format

Problems **06–09** use a different interview format from 01–05, and it
rewards different preparation. Read this before starting any of them.

## The format

```
Pre-work (async, before the call): complete your solution
0–5 min    introduction
5–50 min   building (or expanding) your solution
50–60 min  your questions on the company / tech
```

The prompt you receive is **two sentences**. Something like:

> *Imagine a theoretical or actual system like <Company> which can manage
> stateless and stateful compute workloads. Design the engine for managing
> observability.*

That is the entire brief. There are no numbers, no SLOs, no constraints, no
scale. This is deliberate and it is the first thing being tested.

## What is actually being assessed

Not "did you produce a good architecture." You had a week; everyone produces
a plausible architecture. The 45-minute middle block is where the decision
gets made, and it assesses four things:

**1. Did you establish the problem yourself?**
With no stated scale, a candidate who picks numbers, states them as
assumptions, and derives constraints from them has done the Staff move
before the call starts. A candidate who designed for an unstated scale is
describing a system nobody can evaluate — "it depends how big" is the
correct answer to almost every follow-up they'll receive, and they won't
have it.

**2. Can your solution be extended live?**
"Building (or expanding) your solution" means the interviewer picks a
dimension and pushes. *"What if one customer produces a third of your total
volume?"* *"What happens when that machine dies?"* *"Now make it
multi-region."* You need to be able to change the design in front of them,
out loud, and have the change be coherent with everything you said before.
**This is the trap of the format.** A polished static artifact — a beautiful
diagram, a finished document — is the wrong deliverable, because you can't
edit it live and you'll spend the block defending it instead of building.

**3. Do you know where your design breaks?**
The strongest possible position is having already found the weak point and
being able to say "here is where this falls over, here is the number, and
here are the two things I'd do about it." That converts the interviewer's
best question into your prepared answer.

**4. Do you actually want to work there?**
The last 10 minutes is not a formality and is frequently where infra-heavy
companies form their strongest opinion. See below.

## Preparing an artifact you can extend

**Bring something editable, in a form you can reason over on a screen
share.** Concretely, what works:

- A **markdown doc** with a text diagram, so you can type into it live.
- A **whiteboard-style canvas** (Excalidraw and similar) with your diagram
  already drawn and space around it, so you can add boxes while talking.
- **A small amount of real code or config**, if the prompt invites it —
  a schema, an API sketch, a partitioning function. For an
  engine-design prompt, a data model beats a component diagram.

What doesn't work: slides, a finished PDF, or a diagram so dense that adding
anything requires re-laying it out.

**Structure it so pieces can be swapped.** If your document is a narrative
argument, changing one decision invalidates the next four pages. If it's a
set of components with stated interfaces, plus a separate list of decisions
with their alternatives, you can replace one component live and the rest
still stands.

Have these ready *as separate material you can pull up*, not as prose in the
main doc:

| Have ready | Why |
| --- | --- |
| Your assumed scale, with the derivation | First question, every time |
| 3–5 numbers you can recall exactly | Anchors everything else |
| The alternatives you rejected, and why | "Why not just use X?" is guaranteed |
| The 2–3 places it breaks | Turns their best question into your answer |
| One thing you deliberately did not solve | Scoping is a senior signal |
| A sketch of the 10x version | "What happens at 10x?" is standard |

## Common failure modes

- **Designing before establishing scale.** Everything downstream is
  unevaluable.
- **Bringing a finished thing.** You get 45 minutes of defending rather than
  building, and defending reads as rigidity.
- **Over-broad scope.** The prompt says *engine*. Designing the whole
  platform means every component is shallow. Pick the engine, name what
  you're excluding, and go deep.
- **No numbers.** "It scales horizontally" without a number is the single
  most common weak answer in infrastructure interviews.
- **Reciting a stack.** Naming Kafka, ClickHouse, and Kubernetes is not a
  design. *Why* those, what you'd have to build around them, and what
  they'd cost is the design.
- **Not knowing the company's actual problem.** These prompts come from
  companies solving them for real, right now. If they run their own
  hardware, a design that assumes managed cloud services answers a question
  they didn't ask.
- **Freezing when pushed.** The push is the interview. "Let me think about
  that for a second" and then thinking out loud is a fine answer; going
  quiet or defending reflexively is not.

## The last ten minutes

Prepare 5–6 questions and expect to use 3. Good ones for an
infrastructure-heavy company are specific and show you've thought about
their actual constraints:

- What broke most recently, and what did you change afterwards?
- Where does the current architecture hurt most — what would you rebuild?
- How does on-call work, and what does a typical page look like?
- What is the split between building new capability and operating what
  exists?
- How do decisions like *build vs buy* get made, and who makes them?
- What does this team look like in 12 months?

Weak questions are ones whose answers are on the careers page.

**Do your research on the actual company before the call.** Read their
engineering blog, their status page history, their public postmortems, any
conference talks. For a company that has publicly written about its
infrastructure, arriving without having read it is a visible gap — and the
material usually tells you exactly which constraints they care about, which
makes your design better *and* your questions sharper.

## How to use problems 06–09 in this repo

Each problem gives you the terse prompt as you'd actually receive it,
followed by the context you would have had to establish yourself. Work them
in either of two ways:

**Realistic mode.** Read only the prompt at the top of `BRIEF.md` and stop.
Establish your own scale and constraints, then compare against the brief's
"reference scale" section afterwards. This is the closest simulation and the
most useful.

**Guided mode.** Read the whole brief, take the reference scale as given, and
concentrate on the design. Use this when you want to practise the
architecture rather than the scoping.

Either way: **produce the artifact you would actually bring**, not a
worksheet. The worksheet exists to make sure you've covered the ground; the
artifact is the deliverable, and the drill in Part 10 is the thing that
actually prepares you for the 45 minutes.

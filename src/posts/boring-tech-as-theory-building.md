+++
title = 'Boring tech as (less) Theory Building'
date = 2025-09-04T12:06:09-03:00
draft = false
+++

# Boring tech as (less) Theory Building

[Programming as Theory Building](https://pages.cs.wisc.edu/~remzi/Naur.pdf) by
Peter Naur, is a seminal paper challenging the notion that programming’s
primary output is code. Instead, Naur argues that programmers' primary activity
consists of constructing theory: a living understanding of the system’s design,
constraints and domain. This theory exists only in the minds of developers, not
in the source code, documentation or tests, and is what enables them to debug,
modify and extend their software quickly and effectively. When the theory is
lost (e.g., through team turnover), the program becomes "dead": it may continue
to run correctly forever, but modifying it coherently is near-impossible.

Thus, in Naur's view, code is merely a byproduct; the theory is the program,
and is that which must be preserved. This conceptual framework about the
essence of programming may explain why today’s "boring tech" movement:
prioritizing mature, well-understood tools, isn’t just a pragmatic choice, it's
a necessary one for our software's longevity.

# Boring tech: A definition by negation

Boring tech is best defined by what it avoid: *surprises*. A tool is "boring"
if
* it behaves predictably across different contexts, with minimal
inconsistencies. 
* it is mature (well tested, lots of real-world use)
* It is well-understood, i.e. there are a sufficient number of people that
understand it well enough to use it, debug it, and potentially modify it.

It's important to acknowledge that "boring" is not a definitive property of a
program, rather a program is only boring *relative* to other programs. Since
all programs have small inconsistencies and unexpected behaviors, what differs
is their amount, and the overarching design behind the program.

Take REST and GraphQL, they are both ways of designing web APIs. RESTful APIs
return data in rigid predefined formats, while GraphQL APIs allow the client to
specify different views on the same data, read multiple resources at once, and
offer much more client-side control of the data. GraphQL is obviously more
capable than REST, it can handle streaming, patching and partial responses, all
out of the box. But it’s more complicated than REST, requires more effort to
spin up, maintain and use precisely because it does more things, that’s why 99%
of applications use REST.

# Boring tech through the lens of Theory Building

Naur’s Theory Building view reframes programming as an act of constructing and
maintaining a shared mental model (a “theory”) of how a system works, why it
works that way, and how changes might propagate. This theory isn’t just
abstract: it’s the connective tissue that allows developers to reason about the
system, debug it, and evolve it without breaking coherence. Through this view,
we conclude that boring tech thrives because it is easier to preserve and
transmit its underlying theory, because:

## 1. Complexity erodes theory longevity

Complex systems have complex theories, which are harder to build, maintain and
pass on. Simpler systems endure because their theories are generally smaller,
simpler and overall easier to communicate. When a theory grows too complex, new
developers have more difficulty reconstructing the original reasoning, leading
to hacks that further erode coherence and increase system complexity.

When SQLite decided to not do clustering, for example, it was able to "shrink"
its theory in relation to other SQL databases at the cost of generality, there
is a whole category of problems (and solutions) that SQLite does not concern
itself with. A developer using MongoDB will have to spend significantly more
time and mental effort dealing with sharding, eventual consistency, and so on.
A SQLite user, on the other hand, can be sure that a return from a write means
the data is persisted, and that’s it.

## 2. Coherence reduces surprises

A theory’s predictive power depends on its internal consistency. When
components of a system behave in similar ways under similar conditions,
developers can extrapolate rules from one part of the theory to another. Boring
tech achieves this through design cohesion, applying the same principles
universally. Surprises arise when two components that seem related behave
differently. Cohesive design ensures that once a developer internalizes a core
principle (e.g., “files are streams of bytes”), it applies everywhere. Boring
tech minimizes “theory fractures” (edge cases where the mental model breaks
down) by eliminating inconsistencies. This lets developers focus on what the
system does, not how to navigate its idiosyncrasies.

Unix’s “everything is a file” philosohpy is a great example of how coherence
leads to a simpler mental model. By making everything a file, developers only
need to learn one API to interact with wildly different resources; by using
text as the standard interface between utilities, unix utilities can be
composed to great effect (via piping) without much effort.

## 3. Clear boundaries simplify theory transfer

A system's theory isn't self contained, it also interacts with its
dependencies. Boring tech with clearly defined boundaries reduces the overall
"theory footprint" developers must build and maintain. Good abstractions hide
complexity, allowing developers to focus on their own theories without
absorbing tangential ones. In C, for example, a significant part of the program
theory has to deal with memory: when and where it was created, for how long
it's valid, when it should be destroyed, etc. These concepts are an integral
part of the program's theory, whereas in GC-ed languages they are almost an
afterthought. This lets developers focus their theory building efforts where
they matter: in the core business logic. Garbage collection is not free of
course, and it does incur a runtime cost, but this trade-off is worth it for a
large number of applications (where performance is not the primary concern)
since it "shrinks" their theory significantly.

# Boring Tech avoids incidental complexity

Another way of describing boring tech is how it avoids incidental complexity
(complexity which is not inherent to the problem). By keeping program
complexity as closely tied to problem complexity as possible, two things are
achieved: The first is that overall complexity is minimized. Since incidental
complexity is by definition unnecessary, removing it is obviously a good and
desirable thing. But the second and more important point, is that by learning a
boring technology, one learns the inherent complexities of a particular
problem, and this knowledge is far more applicable and useful than knowing all
the warts and inconsistencies of a particular solution. By internalizing the
theory behind a problem instead of a program, it's much easier to switch to
different solutions if/when needed, and when that happens most of what the
programmer learns can carry over to the next tool.

If you were to learn a lot about react and suddenly switch frameworks, would
your knowledge still apply? Kind of, I guess? Most frameworks are trying to
solve the same problem, but once they start to abstract too much from the
underlying technology, that knowledge becomes less and less transferable. A
person that started using vanilla JS, on the other hand, would be able to apply
most of their knowledge to any of the JS frameworks out there, since they have
a very good understanding of the problem that frameworks are trying to solve.

# Conclusion

Naur’s insight that programs die when their theory fades is a very compelling
explanation for why “boring” tools outlive trendy (non-boring) ones. They
protect the developers’ collective understanding from being diluted by
unnecessary complexity, which not only improves maintanability in the long run,
but also make the knowledge gained from that particular technology much more
general and applicable to other solutions (and similar problems).

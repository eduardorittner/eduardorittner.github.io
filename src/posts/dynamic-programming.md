+++
title = 'How to solve any dynamic programming problem'
date = 2024-05-20T00:01:33Z
draft = false
+++

Dynamic programming is a scary name, but once you understand their general structure of these problems, they become much easier to solve. But first, what is dynamic programming? Dynamic programming is the discipline of solving combinatorial optimization problems as efficiently as possible, without having to calculate every possible solution. In technical terms, an optimal solution is any solution which maximizes (or minimizes) the target function (what we want to optmize). 
There is a 3 step process to solving any DP problem:
1. Define the structure of an optimal solution
2. Define the value of optimal solutions to smaller subproblems (recursively)
3. Compute the value of the optimal solution starting from the smallest possible subproblems.

Let's go through these one by one taking as an example the Backpack problem: Let $I = {1, 2, ..., n}$ be a collection of items, where each item $i$ has a value $v_{i}$ and weight $w_{i}$. Suppose we have a backpack with maximum capacity $W$. The objective is to choose a subset $S\subseteq I$ such that:
- The sum of the item weights in $S$ does not exceed $W$.
- The sum of the item values in $S$ is as high as possible

The first step is to define the structure of an optimal solution to the Backpack problem. Take an arbitrary instance of the problem with $n$ items, maximum capacity $W$ and optimal solution $S$. If we examine the $n_{th}$ element (the last one in the set), there are only 2 possibilities:
1. If $n\notin S$ then $S$ is an optimal solution to the backpack problem with maximum capacity $W$ and items $I-{n}$.
2. If $n\in S$ then $S-{n}$ is an optimal solution to the backpack problem with maximum capacity $W-w_{n}$ and items $I-{n}$. Note that if $w_{n}>W$, then $S$ cannot contain $n$.

Thus, we have succesfuly defined the structure of any optimal solution $S$ to the backpack problem. Note that generally, we start with an optimal solution and then analyze what it means for a particular decision to be made or not (in this case, for an item to be included or not in the final set).

Moving on to step 2, we have to find the recurrence relation which defines the optimal solution to any subproblem. To make our lives easier we should define a way to reference all possible subproblems. In our case, we have 2 variables: The set of items $I$ and the maximum capacity $W$, so we can define every subproblem using only 2 numbers like so: $z[i,c]$ is the optimal sum of the supbroblem with maximum capacity $c$ given only the first $i$ items in $I$.

Now that we have a precise and concise way of referring subproblems, let's start with the simplest of them: What happens when there are no items (i.e. $i=0$)? The best possible answer is clearly 0. And what about when the maximum capacity is 0 ($c=0$)? The best possible answer is also 0.
Next up, we know from the structure of an optimal solution that if $z[i,c]$ uses the item $i$, then $z[i,c]=v_{i}+z[i-1,c-w_{i}]$. Similarly, if $z[i,c]$ does not use the item $i$, then $z[i,c]=z[i-1,c]$. Having covered all these cases, we can now define the generic value of any solution $z[i,c]$ like so:
1. $z[0,c]=z[0,c]=0$
2. $z[i,c]=max(z[i-1,c], z[i-d,c-w_{i}]+v_{i})$

Now all that's left is to compute the optimal solution with the recurrence relation we have found. A simple algorithm to do this would be to construct a $n$x$W$ matrix where the value in $m[i, j]$ is the solution to the subproblem of the first $i$ items and maximum capacity $j$, and fill it out until we reach the value of $m[n,W]$, which is the answer we want. To fill the matrix out is pretty simple, we first start by filling in all the base cases, where $i=0$ or $j=0$, and after that we fill out each column starting from the second onwards.

And there we go, we have solved the Backpack problem using a simple DP solution. Let's review the steps we took one by one:
First, we defined the structure of an optimal solution, this means that we take an arbitrary optimal solution S, and find out what it means to make a certain choice, in our case, the decision is whether to include an item or not.

Second, we define the value of optimal solutions recursively, to do this we first define a way to reference any subproblem, then define the base cases, which are the subproblems whose answers are obvious and always the same, like when there are no items to choose from, or a maximum capacity that's zero. After that we can define the answer for any general subproblem, which will be the maximum (or minimum, depending on the problem) between its possible answers.

And finally, we define an algorithm that calculates the recurrence relation we found previously. There are often several different algorithms that can be designed from the same recurrence relation, and the most optimized algorithms always have to take into account the specifics of the problema at hand. However, most of the time one can construct a reasonably efficient algorithm by simply calculating all the values inside the matrix of subproblems, starting from the base cases up until the problem we actually want to answer.

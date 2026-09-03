# SONAR Modernization

## Overview

You've inherited a package that classifies and detects underwater objects. How cool!

You are resuming development of this capability to meet the needs of today's submarine captains, so it can once again be used in active duty vessels.
However, the code hasn't been updated in decades and it does not use modern software engineering best practices.

## Task

Your primary task is to modernize the legacy SONAR package by re-implementing its functionality in the `sonar-modern` directory, adhering to modern software engineering best practices.

We want our submarine captain to feel confident that the code will work reliably and correctly in mission-critical situations.
At the same time, our dev team must be able to efficiently understand, maintain, and extend this package as future mission requirements evolve.
Your submission should reflect what you feel are the best ways to address these priorities.

> [!TIP]
> We're looking for production ready software that incorporates best-practices for maintainability and development. Consider aspects that contribute to a professional, robust, and easily deployable software package. Don't worry that you're over-engineering it. If you'd want it as part of production code you maintain, then add it!

## Project Structure

We've structured the codebase as a monorepo with legacy code and modernized code side-by-side:

```shell
.
└── packages/
    ├── sonar-legacy/
    │   └── …
    └── sonar-modern/
        └── …
```

You may add and edit files anywhere in the project directory structure as necessary for any tooling you choose to use, but place your modernized implementation of the SONAR application in the `packages/sonar-modern` package directory.

## Language Requirements

The legacy package is written in C.
Write your modern package in either Python or Rust.

## Development Process

You are free to leverage AI tools (eg., Cursor, Copilot, Chat GPT) to assist in your development process. However, the final submission should demonstrate your personal quality standards and engineering decisions. We are interested in your approach to building robust software, not just the output of an AI.

Treat this as though it were an open-source package being built by you in public.

Godspeed.

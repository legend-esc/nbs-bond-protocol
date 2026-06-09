# Agentic IDE Prompts — NbS Bond Protocol

Each `day-XX-*.md` file is a self-contained prompt for an agentic IDE (Cursor, Windsurf, Copilot Agent mode, etc.).

## How to Use

1. **Load context first.** Before running any day prompt, load `context/tech-stack.md` into the agent so it understands the project conventions.
2. **One day at a time.** Execute prompts sequentially — each builds on the previous.
3. **Verify after each day.** Run the test commands listed in each prompt's "Verification" section before moving on.
4. **Commit.** Each prompt ends with a suggested commit message.

## Prompt Structure

| Section | Purpose |
|---------|---------|
| **Goal** | What the agent should accomplish |
| **Files to Create** | Exact paths the agent must write |
| **Technical Spec** | Types, signatures, logic — the contract |
| **Edge Cases** | Off-nominal paths the agent must handle |
| **Verification** | Commands to confirm correctness |
| **Commit Message** | git commit to use |

## File Naming Convention

```
day-XX-description.md
```

Where `XX` = day number (01–16).

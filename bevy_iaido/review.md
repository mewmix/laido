# Gameplay Review & Exploit Analysis

## AI Tuning
The AI difficulty has been adjusted to allow for easier testing of win and parry conditions.
- **DUMB Profile**: Mean Reaction: 500ms, Wrong Rate: 30%. (Currently Active)
- **NOVICE Profile**: Mean Reaction: 450ms, Wrong Rate: 20%.
- **SKILLED Profile**: Mean Reaction: 350ms, Wrong Rate: 10%.
- **MASTER Profile**: Mean Reaction: 140ms, Wrong Rate: 0%.

## Exploit Vectors

### 1. Undefined Matchups
- **Issue**: Each input only explicitly beats 2 and loses to 2. The other 5 relationships are "Neutral".
- **Current Resolution**: Neutral matchups are resolved by **Speed Check**. Faster reaction wins.
- **Risk**: If reaction windows are generous, neutral matchups become a coin flip or purely ping-dependent in multiplayer. Against AI, it becomes a pure reaction test. Players might gravitate towards inputs that are statistically more likely to be Neutral to force a speed test if they are confident in their reaction time.

### 2. Parry Farming
- **Issue**: Same input = Parry/Clash. Clashes reset the round with a shorter timer.
- **Risk**: A player can intentionally mirror the opponent (or the prompt, if mirroring is the strategy) to force clashes repeatedly.
- **Mitigation**: Clashes reduce the input window (`CLASH_INPUT_WINDOW_MS`), making subsequent parries harder. However, if the player is just reacting to the prompt, and the AI is also reacting to the prompt (mirroring), high-level play might devolve into endless clashes until someone errors.

### 3. Unreachable Combos
- **Issue**: `UP+DOWN` and `LEFT+RIGHT` are physically impossible with a single standard swipe.
- **Status**: These inputs are technically allowed via keyboard combos (pressing both keys).
- **Risk**: These inputs might be "safe" if the opponent cannot physically perform them to counter, or "useless" if they are hard to execute. If the AI can use them, the player might feel cheated if they can't easily reciprocate without a keyboard.

### 4. Input-Effort Advantage
- **Issue**: Single keys (Up, Down) are faster to press than combos (Up+Right).
- **Risk**: In a Neutral matchup resolved by Speed, the player using a single key has a physical advantage (less finger movement/coordination). This encourages "camping" on single directions.

### 5. Predictable Local Meta
- **Issue**: The 10-node wheel encourages picking moves that beat the most common moves.
- **Mitigation**: The current implementation enforces a **Strict Simon Says** mechanic (Opening determines the ONLY correct input). This removes the "Meta" choice entirely, reducing the game to Reaction Speed + Input Precision.
- **Pivot**: If the design goal is *choice* (RPS), the "Correct Input" constraint needs to be relaxed to allow ANY valid counter, not just the Mirror. Currently, `correct_direction_for` enforces a 1:1 mapping.

## Next Steps
- **Decide on Gameplay Loop**: 
    - **Option A (Current)**: Strict Reaction. See X, Input X. RPS only resolves speed ties or wrong inputs (which are instant losses anyway).
    - **Option B (Strategic)**: See Stance X. You must Input Y (where Y beats X).
    - **Option C (Freeform)**: See Stance X. You can input anything. If you beat X, you win. If you lose to X, you lose. If Neutral, Speed. (This allows camping).

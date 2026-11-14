---
status: draft
---

# Workout formula

How a finished game becomes a workout. Used by [[Post-game workout]]. Implemented as a pure function in `cooldown-core` (game stats + role → workout), so every rule below gets a unit test.

All numbers are **starting defaults to tune after real games**, not final values.

## Principle
Penalty only (see [[Vision]]): each exercise pays for one thing the player gave away. Doing better than the target gives zero for that exercise, never a negative amount.

Each role has one **main stat**, the thing that role is judged on first. It drives the most complete exercise.

| Exercise | Unit | Pays for |
|---|---|---|
| Burpees | reps | The role's **main stat** under target |
| Squats | reps | The role's **secondary stat** under target |
| Plank | seconds | Deaths (every role) |

## Inputs
From the end-of-game stats of the local player (see [[Post-game workout#Data]]), or, for a game imported from the client's match history, from that history (see [[History#Games from before cooldown.lol]]):
- `minutes` = game length in seconds / 60
- `deaths`
- `cs` = minions killed + neutral minions killed
- `damage_share` = player damage to champions / team damage to champions
- `kill_participation` = (kills + assists) / team kills (0 if the team has no kill)
- `vision_score`

## Role stats and targets
| Role | Main stat (burpees) | Target | Secondary stat (squats) | Target |
|---|---|---|---|---|
| Top | CS / min | 7.0 | damage share | 22 % |
| Jungle | kill participation | 60 % | CS / min | 5.5 |
| Mid | damage share | 25 % | CS / min | 7.5 |
| Bottom | CS / min | 8.0 | damage share | 28 % |
| Support | vision score / min | 2.0 | kill participation | 55 % |

## Gap per stat
How far under target the player is, never below 0:

| Stat | Gap |
|---|---|
| CS / min | missed CS = CS target × `minutes` − `cs` |
| vision score / min | missed vision = vision target × `minutes` − `vision_score` |
| damage share, kill participation | percentage points under target |

## Rules
1. **Burpees** (main stat), the heaviest exercise, so fewer reps per unit of gap:
   - CS: 1 per 5 missed CS
   - vision: 1 per 2 missed vision points
   - damage share: 2 per point
   - kill participation: 1 per 2 points
2. **Squats** (secondary stat):
   - CS: 1 per 2 missed CS
   - damage share: 5 per point
   - kill participation: 1 per point
3. **Plank** = 10 seconds × `deaths`.
4. Every amount is rounded up.
5. **Caps per game**: burpees 30, squats 50, plank 180 s. When a cap applies, the UI says so.
6. **No workout** when the game is a remake (shorter than 5 minutes, or flagged as a remake by the client).

## Worked example
Mid, 7/6/4, 168 CS, 22 % damage share, 28 minutes:

| Exercise | Computation | Amount |
|---|---|---|
| Burpees | damage share 25 % − 22 % = 3 points × 2 | 6 |
| Squats | 7.5 × 28 = 210 target CS, 210 − 168 = 42 missed, 42 / 2 | 21 |
| Plank | 6 deaths × 10 s | 60 s |

## Open questions
- Should targets depend on rank or be set by the player?
- A difficulty multiplier setting (×0.5 / ×1 / ×1.5)? Needs settings persistence ([[Roadmap]]).
- Win modifier (see [[Vision#Open questions]]).

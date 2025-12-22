# AI Game Director Design (The "Dungeon Master" Engine)

## Overview
This document outlines the architecture for an **AI-Driven Dynamic RPG Engine**.
Instead of static difficulty or scripted events, an **LLM-based Game Director** observes the player's actions in real-time and modifies the game world to optimize for "Engagement" and "Fun".

**Concept:** Think of it as a virtual *Dungeons & Dragons* Dungeon Master (DM) who watches you play and secretly adjusts the HP of the boss, drops a healing potion when you're dying, or spawns a surprise ambush if you look bored.

---

## 1. Architecture: The "OODA" Loop

The system follows the **Observe -> Orient -> Decide -> Act** loop.

### 1.1 Observe: Player Telemetry (The "Eyes")
We need to capture *meaningful* data, not just raw stats.
*   **Combat Metrics:** DPS, Damage Taken/Sec, Potion Usage Rate, Accuracy.
*   **Behavioral Metrics:**
    *   *Exploration Rate:* Is the player rushing or checking every corner?
    *   *Idle Time:* Is the player reading lore or AFK?
    *   *Frustration Indicators:* Re-loading saves frequently, spamming buttons, dying repeatedly to the same mob.
*   **Emotional State (inferred):** "Bored" (High kill rate, zero damage taken), "Stressed" (Low HP, panic rolling), "Flow" (Steady progress).

### 1.2 Orient: Context Construction (The "Prompt")
We condense 10 minutes of gameplay into a concise JSON context for the LLM.
```json
{
  "player_status": { "hp": "20%", "class": "Mage", "stress_level": "High" },
  "recent_events": ["Died to Goblin King", "Respawned", "Failed jump puzzle 3 times"],
  "current_objective": "Reach the Castle",
  "world_state": { "weather": "Sunny", "difficulty": 1.2 }
}
```

### 1.3 Decide: The LLM Brain
We ask the LLM (e.g., GPT-4o, Claude 3.5, or local Llama 3):
*"Given the player is stressed and failing repeatedly, what should we change to keep them engaged but challenged?"*

**LLM Response (Structured Command):**
```json
{
  "action": "AdjustDifficulty",
  "parameters": {
    "enemy_aggro_radius_multiplier": 0.8,
    "drop_loot": "HealthPotion",
    "trigger_event": "NarrativeHint"
  },
  "reasoning": "Player is frustrated. Lowering difficulty slightly and offering aid to prevent churn."
}
```

### 1.4 Act: World Mutators (The "Hands")
The Engine receives the command and executes it via **ECS Systems**.
*   **Spawning System:** Spawn generic enemies or specific "Helpful NPCs".
*   **Stats Mutator:** Dynamically scale `Enemy.Damage` or `Enemy.HP` in real-time.
*   **Environment Controller:** Change weather (Rain -> Clear Sky to improve mood), open shortcuts.
*   **Narrative Engine:** Generate dynamic quest text (e.g., a note on the ground giving a hint).

---

## 2. Implementation Details

### 2.1 The "Director" ECS System
A global system running on the Server (or Client for single player).

*   **Frequency:** LLM calls are expensive/slow. Do NOT run every frame.
    *   *Pulse Check:* Every 30-60 seconds.
    *   *Trigger check:* On player death, On quest complete, On entering new zone.
*   **Cost Management:**
    *   Use exact telemetry for logic (Logic-based DDA) for 90% of cases.
    *   Call LLM only for **Complex Decision Making** or **Narrative Generation**.

### 2.2 Dynamic Quest Generation
The AI can write quests tailored to playstyle.
*   *If Player = "Genocidal Maniac"*: Quest -> "Slay 50 Bandits for the Blood God".
*   *If Player = "Pacifist/Stealth"*: Quest -> "Steal the document without being seen".

### 2.3 The "Fun" Function
How does the AI know if the player is having fun?
*   **Implicit Feedback:** Continued playtime, engagement depth.
*   **Explicit Feedback:** Optional "Rate this Quest" prompt (risky, breaks immersion).
*   **Flow State Target:** Design the AI to aim for the "Goldilocks Zone" (Challenge level matches Player Skill).

---

## 3. Risks & safeguards

### 3.1 "The Ghost in the Machine"
*   **Risk:** AI spawns an unkillable boss or breaks the game logic.
*   **Mitigation:** **Sanity Layers**. The LLM outputs *intent*, but the Engine validates it.
    *   *Rule:* Cannot spawn enemy level > player level + 5.
    *   *Rule:* Cannot remove Key Items.

### 3.2 Latency
*   **Risk:** LLM takes 3 seconds to reply.
*   **Mitigation:** Asynchronous execution. The Director plans ahead. It prepares the "Ambush" 10 seconds before the player reaches the spot.

---

## 4. Tech Stack Recommendation
*   **Local LLM (On-Device):** quantization models (Llama-3-8B-4bit). Zero latency, free, works offline. Good for simple logic.
*   **Cloud LLM (API):** GPT-4o. High cost, latency. Use only for complex dialogue generation.

## 5. Roadmap
1.  **Metric Logger:** Build the telemetry recorder system.
2.  **Director API:** Create the interface for "Mutators" (Code that changes game settings).
3.  **Rule-Based DDA:** Implement simple "If dying -> reduce HP" logic first (Baseline).
4.  **LLM Integration:** Connect the "Brain" to replace the simple rules.

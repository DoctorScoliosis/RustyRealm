# Section 25 — Companion Discovery and Roster Visibility

**Status:** Implemented in PR #29 on \`work/section-20-realm-progression\`.

## Persisted lifecycle
- \`GameState.discovered_companions\` stores stable \`CompanionId\` values independently of recruited \`CompanionState\` records and active \`Party\` membership.
- Discovery is idempotent and does not recruit or assign a companion.
- Discovered-but-unrecruited companions appear as known opportunities; recruitment still uses the existing level and gold validation.
- Undiscovered definitions are hidden from roster, party and recovery surfaces.
- Legacy saves reveal only companions already present in their recruited roster. They do not reveal every static definition.

## Gameplay discovery hooks
| Existing evidence | Revealed lead |
|---|---|
| Oakhaven \`clear_the_ruins\` quest stage | Bran the Shield |
| Crumbling Watchtower scout report | Lyra Swiftbow |
| Standing Stones survey | Maren Starweave |
| Sunken Ruins signature encounter | Garrick Ironfist |
| Whispering Woods patrol | Sera Nightwhisper |
| Goblin Caves expedition | Torval Embershield |

These hooks reuse Section 19's stable quest-stage, landmark and encounter IDs. Repeated events do not duplicate discoveries or auto-recruit companions.

## Validation
Tests cover fresh-save secrecy, idempotent discovery, individual visibility, migration of already-recruited legacy companions, and discovery without recruitment. Party and recovery views filter through the discovered roster.

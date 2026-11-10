# ask-many

Several yes/no questions on one card, answered in any order, then one Send: a single wake `{ctx, answers: {id: "yes"|"no"}}`, so a batch of
decisions costs the agent one turn. Layout: `cache/seed/ask-many.json`, component `ui/components/asks/`.

**Data:** `{"title": "Before I push", "ctx": "pre-push", "items": [{"id": "commit", "q": "Commit the fix?"}, {"id": "close", "q": "Close the test card?"}]}`
**Push:** `morc use ask-many --data @file.json`. Send stays disabled until every question has an answer; the card closes after Send.

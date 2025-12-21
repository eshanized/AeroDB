
No other transitions are permitted.

---

## 7. Forbidden Transitions (Explicit)

The following transitions are **forbidden**:

- `Steady → AuthorityTransitioning`
- `PromotionRequested → PromotionApproved`
- `PromotionValidating → AuthorityTransitioning`
- `PromotionDenied → AuthorityTransitioning`
- Any transition driven by timeouts or retries
- Any implicit re-entry into `PromotionApproved` after crash

---

## 8. Crash Semantics per State

| State | Crash Outcome |
|-----|--------------|
| Steady | No effect |
| PromotionRequested | Promotion forgotten |
| PromotionValidating | Promotion forgotten |
| PromotionApproved | Promotion forgotten |
| AuthorityTransitioning | Atomic outcome enforced |
| PromotionSucceeded | Authority preserved |
| PromotionDenied | Promotion forgotten |

Crash behavior is deterministic and invariant-preserving.

---

## 9. Observability Requirements

Each state transition MUST emit:
- State entry event
- Transition reason
- Relevant invariant references

Silent transitions are forbidden.

---

## 10. State Machine Completeness Rule

This state machine is complete when:
- Every promotion attempt follows exactly one path
- No ambiguity exists after crash
- All invariant violations result in `PromotionDenied`
- All success paths converge to `Steady`

---

END OF DOCUMENT
